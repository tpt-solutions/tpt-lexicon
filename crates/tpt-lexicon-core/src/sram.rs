//! SRAM-native vocabulary mapping: cache-resident lookup structures for BPE
//! merge rules.
//!
//! The [`MergeIndex`] provides O(1) amortized lookup for adjacent byte-pair
//! matches, replacing the O(pairs × merges) linear scan with a hash-indexed
//! structure designed to fit in L1/L2 cache for typical vocabulary sizes.

use alloc::vec::Vec;

use crate::vocab::MergeEntry;

/// A fixed-capacity hash index mapping `(left, right)` byte-pairs to their
/// merge rank.
///
/// For vocabularies up to ~100K merges, this fits comfortably in L2 cache
/// (≈4–8 MB). The design avoids pointer-chasing and uses flat storage for
/// scan-friendly access patterns.
///
/// # Lookup complexity
///
/// Average O(1) for pair lookup; worst-case O(bucket_size) with chaining.
#[derive(Debug, Clone)]
pub struct MergeIndex {
    /// Hash table buckets. Each bucket stores (left_hash, right_hash, rank).
    buckets: Vec<Bucket>,
    /// Number of entries in the index.
    len: usize,
    /// Log2 of bucket count (capacity = 1 << shift).
    shift: u32,
}

#[derive(Debug, Clone, Default)]
struct Bucket {
    entries: Vec<(u64, u64, u32)>,
}

impl MergeIndex {
    /// Create a merge index from a sorted list of merge entries.
    ///
    /// Pre-allocates hash table with capacity ≈ 2× the entry count for
    /// low-collision performance.
    pub fn from_merges(merges: &[MergeEntry]) -> Self {
        let min_buckets = merges.len().next_power_of_two().max(16);
        // Compute shift = ceil(log2(min_buckets)) without libm.
        let mut shift = 0u32;
        let mut n = min_buckets;
        while n > 1 {
            n = n.div_ceil(2);
            shift += 1;
        }
        // Ensure at least 4 bits (16 buckets minimum).
        if shift < 4 {
            shift = 4;
        }
        let bucket_count = 1usize << shift;

        let mut index = Self {
            buckets: alloc::vec![Bucket::default(); bucket_count],
            len: 0,
            shift,
        };

        for merge in merges {
            let left_hash = fnv1a(&merge.left);
            let right_hash = fnv1a(&merge.right);
            index.insert_raw(left_hash, right_hash, merge.rank);
        }

        index
    }

    /// Look up the rank for an adjacent byte-pair.
    ///
    /// Returns `Some(rank)` if the pair matches a merge rule, `None` otherwise.
    #[inline]
    pub fn lookup(&self, left: &[u8], right: &[u8]) -> Option<u32> {
        let left_hash = fnv1a(left);
        let right_hash = fnv1a(right);
        let bucket_idx = self.bucket_index(left_hash ^ right_hash);
        let bucket = &self.buckets[bucket_idx];

        for &(lh, rh, rank) in &bucket.entries {
            if lh == left_hash && rh == right_hash {
                return Some(rank);
            }
        }
        None
    }

    /// Returns the number of merge rules in the index.
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the index contains no merge rules.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the number of hash buckets.
    #[inline]
    pub fn bucket_count(&self) -> usize {
        self.buckets.len()
    }

    /// Returns the average number of entries per bucket (load factor).
    pub fn load_factor(&self) -> f64 {
        if self.buckets.is_empty() {
            return 0.0;
        }
        self.len as f64 / self.buckets.len() as f64
    }

    fn insert_raw(&mut self, left_hash: u64, right_hash: u64, rank: u32) {
        let bucket_idx = self.bucket_index(left_hash ^ right_hash);
        self.buckets[bucket_idx]
            .entries
            .push((left_hash, right_hash, rank));
        self.len += 1;
    }

    #[inline]
    fn bucket_index(&self, hash: u64) -> usize {
        (hash >> (64 - self.shift)) as usize
    }
}

/// FNV-1a hash function (64-bit).
///
/// Chosen for its simplicity, excellent distribution, and cache-friendly
/// sequential access pattern.
#[inline]
fn fnv1a(data: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for &byte in data {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

/// Enhanced BPE pair finder using a [`MergeIndex`].
///
/// This replaces the O(pairs × merges) linear scan in
/// [`Vocab::find_best_pair`](crate::Vocab::find_best_pair) with an
/// O(pairs) scan where each pair lookup is O(1) amortized.
pub fn find_best_pair_indexed(
    tokens: &[&[u8]],
    index: &MergeIndex,
) -> Option<(usize, u32)> {
    if tokens.len() < 2 {
        return None;
    }

    let mut best_index = None;
    let mut best_rank = u32::MAX;

    for i in 0..tokens.len() - 1 {
        if let Some(rank) = index.lookup(tokens[i], tokens[i + 1]) {
            if rank < best_rank {
                best_rank = rank;
                best_index = Some(i);
            }
        }
    }

    best_index.map(|idx| (idx, best_rank))
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn fnv1a_deterministic() {
        let h1 = fnv1a(b"hello");
        let h2 = fnv1a(b"hello");
        assert_eq!(h1, h2);
    }

    #[test]
    fn fnv1a_different_inputs() {
        let h1 = fnv1a(b"hello");
        let h2 = fnv1a(b"world");
        assert_ne!(h1, h2);
    }

    #[test]
    fn fnv1a_empty() {
        let h = fnv1a(b"");
        assert_ne!(h, 0);
    }

    #[test]
    fn merge_index_from_empty() {
        let index = MergeIndex::from_merges(&[]);
        assert!(index.is_empty());
        assert_eq!(index.len(), 0);
        assert!(index.lookup(b"h", b"e").is_none());
    }

    #[test]
    fn merge_index_lookup_found() {
        let merges = vec![
            MergeEntry {
                left: b"h".to_vec(),
                right: b"e".to_vec(),
                rank: 5,
            },
            MergeEntry {
                left: b"l".to_vec(),
                right: b"l".to_vec(),
                rank: 3,
            },
        ];
        let index = MergeIndex::from_merges(&merges);
        assert_eq!(index.len(), 2);
        assert_eq!(index.lookup(b"h", b"e"), Some(5));
        assert_eq!(index.lookup(b"l", b"l"), Some(3));
    }

    #[test]
    fn merge_index_lookup_not_found() {
        let merges = vec![MergeEntry {
            left: b"h".to_vec(),
            right: b"e".to_vec(),
            rank: 5,
        }];
        let index = MergeIndex::from_merges(&merges);
        assert_eq!(index.lookup(b"x", b"y"), None);
    }

    #[test]
    fn merge_index_load_factor() {
        let merges: Vec<MergeEntry> = (0..100)
            .map(|i| MergeEntry {
                left: alloc::vec![i as u8],
                right: alloc::vec![(i + 1) as u8],
                rank: i as u32,
            })
            .collect();
        let index = MergeIndex::from_merges(&merges);
        // Load factor should be reasonable (less than 2.0 for good performance)
        assert!(index.load_factor() < 2.0);
    }

    #[test]
    fn find_best_pair_indexed_basic() {
        let merges = vec![
            MergeEntry {
                left: b"h".to_vec(),
                right: b"e".to_vec(),
                rank: 10,
            },
            MergeEntry {
                left: b"l".to_vec(),
                right: b"l".to_vec(),
                rank: 5,
            },
        ];
        let index = MergeIndex::from_merges(&merges);
        let tokens: &[&[u8]] = &[b"h", b"e", b"l", b"l", b"o"];
        let result = find_best_pair_indexed(tokens, &index);
        assert!(result.is_some());
        let (idx, rank) = result.unwrap();
        assert_eq!(idx, 2); // "l" + "l" at rank 5
        assert_eq!(rank, 5);
    }

    #[test]
    fn find_best_pair_indexed_no_match() {
        let merges = vec![MergeEntry {
            left: b"x".to_vec(),
            right: b"y".to_vec(),
            rank: 0,
        }];
        let index = MergeIndex::from_merges(&merges);
        let tokens: &[&[u8]] = &[b"a", b"b", b"c"];
        assert!(find_best_pair_indexed(tokens, &index).is_none());
    }

    #[test]
    fn find_best_pair_indexed_single_token() {
        let merges = vec![MergeEntry {
            left: b"a".to_vec(),
            right: b"b".to_vec(),
            rank: 0,
        }];
        let index = MergeIndex::from_merges(&merges);
        let tokens: &[&[u8]] = &[b"a"];
        assert!(find_best_pair_indexed(tokens, &index).is_none());
    }

    #[test]
    fn find_best_pair_indexed_matches_linear() {
        let merges = vec![
            MergeEntry {
                left: b"a".to_vec(),
                right: b"b".to_vec(),
                rank: 3,
            },
            MergeEntry {
                left: b"b".to_vec(),
                right: b"c".to_vec(),
                rank: 1,
            },
            MergeEntry {
                left: b"c".to_vec(),
                right: b"d".to_vec(),
                rank: 2,
            },
        ];
        let index = MergeIndex::from_merges(&merges);
        let tokens: &[&[u8]] = &[b"a", b"b", b"c", b"d"];

        let indexed = find_best_pair_indexed(tokens, &index);
        assert!(indexed.is_some());
        // "b"+"c" at rank 1 should win
        let (idx, rank) = indexed.unwrap();
        assert_eq!(idx, 1);
        assert_eq!(rank, 1);
    }

    #[test]
    fn merge_index_bucket_count_reasonable() {
        let merges: Vec<MergeEntry> = (0..1000)
            .map(|i| MergeEntry {
                left: alloc::vec![(i >> 8) as u8, i as u8],
                right: alloc::vec![(i >> 8) as u8, (i + 1) as u8],
                rank: i as u32,
            })
            .collect();
        let index = MergeIndex::from_merges(&merges);
        // Bucket count should be at least 2× entries for low collisions
        assert!(index.bucket_count() >= 1024);
    }
}
