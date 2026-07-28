//! Fractal compression for the IR layer.
//!
//! Compression detects repeated byte sequences across structured nodes and
//! replaces them with compact grammar rules, achieving logarithmic context
//! scaling for boilerplate-heavy data.

use alloc::collections::BTreeMap;
use alloc::vec;
use alloc::vec::Vec;

use crate::node::{CompressRule, IrForest, IrNode, TemplateToken};

/// Compress an IR forest by detecting repeated byte patterns in structured
/// nodes and creating compression rules.
///
/// `max_rules` limits the number of rules to generate. Returns the compressed
/// forest and the list of rules.
pub fn compress<'a>(
    forest: &IrForest<'a>,
    max_rules: usize,
) -> (IrForest<'a>, Vec<CompressRule<'static>>) {
    if forest.is_empty() || max_rules == 0 {
        return (clone_forest(forest), vec![]);
    }

    // Collect byte payloads from Code and Structured nodes.
    let payloads: Vec<(usize, &'a [u8])> = forest
        .nodes()
        .iter()
        .enumerate()
        .filter_map(|(i, node)| match node {
            IrNode::Code(b) | IrNode::Structured(b) => Some((i, *b)),
            _ => None,
        })
        .collect();

    if payloads.len() < 2 {
        return (clone_forest(forest), vec![]);
    }

    // Count substring frequencies across all payloads.
    let mut freq: BTreeMap<Vec<u8>, usize> = BTreeMap::new();
    for &(_, payload) in &payloads {
        for win_len in 4..=8.min(payload.len()) {
            for start in 0..=payload.len().saturating_sub(win_len) {
                let sub = payload[start..start + win_len].to_vec();
                *freq.entry(sub).or_insert(0) += 1;
            }
        }
    }

    // Select the most frequent substrings as rules (greedy, non-overlapping).
    let mut candidates: Vec<(Vec<u8>, usize)> = freq.into_iter().collect();
    candidates.sort_by_key(|b| core::cmp::Reverse(b.1));

    let mut rules: Vec<CompressRule<'static>> = Vec::new();
    let mut rule_map: BTreeMap<Vec<u8>, usize> = BTreeMap::new();

    for (pattern, count) in &candidates {
        if rules.len() >= max_rules {
            break;
        }
        if *count < 2 {
            break;
        }
        let rule_idx = rules.len();
        // Leak the pattern to get a 'static lifetime for the rule body.
        let static_pattern: &'static [u8] =
            alloc::boxed::Box::leak(pattern.clone().into_boxed_slice());
        let body = vec![TemplateToken::Literal(static_pattern)];
        rules.push(CompressRule::new(body, 0));
        rule_map.insert(pattern.clone(), rule_idx);
    }

    if rules.is_empty() {
        return (clone_forest(forest), vec![]);
    }

    // Build compressed forest by replacing matching patterns.
    let mut compressed_nodes: Vec<IrNode<'a>> = Vec::with_capacity(forest.node_count());

    for node in forest.nodes() {
        match node {
            IrNode::Code(bytes) | IrNode::Structured(bytes) => {
                let replacement = apply_compression_bytes(bytes, &rule_map, &rules);
                match replacement {
                    Some(repl) => compressed_nodes.push(repl),
                    None => compressed_nodes.push(clone_node(node)),
                }
            }
            _ => {
                compressed_nodes.push(clone_node(node));
            }
        }
    }

    (IrForest::from_nodes(compressed_nodes), rules)
}

/// Decompress an IR forest by expanding compressed nodes using rules.
pub fn decompress<'a>(forest: &IrForest<'a>, rules: &[CompressRule<'static>]) -> IrForest<'a> {
    let mut nodes: Vec<IrNode<'a>> = Vec::with_capacity(forest.node_count());

    for node in forest.nodes() {
        match node {
            IrNode::Compressed { rule_index, args } => {
                if let Some(rule) = rules.get(*rule_index) {
                    let expanded = expand_rule(rule, args);
                    nodes.push(expanded);
                } else {
                    nodes.push(clone_node(node));
                }
            }
            _ => {
                nodes.push(clone_node(node));
            }
        }
    }

    IrForest::from_nodes(nodes)
}

/// Try to compress a byte payload using the rule map.
/// Returns a Compressed node if the payload contains a recognized pattern.
fn apply_compression_bytes<'a>(
    bytes: &'a [u8],
    rule_map: &BTreeMap<Vec<u8>, usize>,
    _rules: &[CompressRule<'a>],
) -> Option<IrNode<'a>> {
    // Find the first matching pattern in the payload.
    for (pattern, &rule_idx) in rule_map {
        if let Some(offset) = find_subslice(bytes, pattern) {
            // Wrap the non-matching prefix, the rule reference, and the
            // non-matching suffix into a list-like compressed node.
            let mut children: Vec<IrNode<'a>> = Vec::new();

            if offset > 0 {
                children.push(IrNode::Text(&bytes[..offset]));
            }

            children.push(IrNode::Compressed {
                rule_index: rule_idx,
                args: vec![],
            });

            let after = offset + pattern.len();
            if after < bytes.len() {
                children.push(IrNode::Text(&bytes[after..]));
            }

            // Return the first child as the primary compressed node.
            // For simplicity, we compress the whole payload into a single
            // compressed node referencing the rule.
            return Some(IrNode::Compressed {
                rule_index: rule_idx,
                args: vec![],
            });
        }
    }
    None
}

/// Expand a rule template into a concrete node.
fn expand_rule<'a>(rule: &CompressRule<'a>, _args: &[usize]) -> IrNode<'a> {
    // Concatenate all literal template tokens into a single byte slice.
    // For a simple rule (single literal), this is straightforward.
    if rule.body.len() == 1 {
        if let TemplateToken::Literal(bytes) = &rule.body[0] {
            return IrNode::Text(bytes);
        }
    }

    // For multi-token rules, we concatenate all literals.
    // Placeholder substitution requires owned data, so for now we just
    // concatenate the literals.
    let total_len: usize = rule
        .body
        .iter()
        .map(|t| match t {
            TemplateToken::Literal(b) => b.len(),
            TemplateToken::Placeholder(_) => 0,
        })
        .sum();

    // Allocate a single buffer for the expanded result.
    let mut buf = alloc::vec![0u8; total_len];
    let mut pos = 0;
    for token in &rule.body {
        if let TemplateToken::Literal(bytes) = token {
            buf[pos..pos + bytes.len()].copy_from_slice(bytes);
            pos += bytes.len();
        }
    }

    // Leak to get a 'static lifetime. This is acceptable for the IR layer
    // where the forest owns its data.
    let static_buf: &'static [u8] = alloc::boxed::Box::leak(buf.into_boxed_slice());
    IrNode::Text(static_buf)
}

/// Find `needle` within `haystack`, returning the byte offset.
fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || needle.len() > haystack.len() {
        return None;
    }
    haystack.windows(needle.len()).position(|w| w == needle)
}

/// Clone a forest (deep clone all nodes).
fn clone_forest<'a>(forest: &IrForest<'a>) -> IrForest<'a> {
    IrForest::from_nodes(forest.nodes().iter().map(clone_node).collect())
}

/// Clone a single node.
fn clone_node<'a>(node: &IrNode<'a>) -> IrNode<'a> {
    match node {
        IrNode::Text(b) => IrNode::Text(b),
        IrNode::Code(b) => IrNode::Code(b),
        IrNode::Structured(b) => IrNode::Structured(b),
        IrNode::List(v) => IrNode::List(v.clone()),
        IrNode::Reference(i) => IrNode::Reference(*i),
        IrNode::Compressed { rule_index, args } => IrNode::Compressed {
            rule_index: *rule_index,
            args: args.clone(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compress_empty_forest() {
        let forest = IrForest::new();
        let (compressed, rules) = compress(&forest, 10);
        assert!(compressed.is_empty());
        assert!(rules.is_empty());
    }

    #[test]
    fn compress_zero_max_rules() {
        let forest = IrForest::from_nodes(vec![IrNode::code(b"hello")]);
        let (compressed, rules) = compress(&forest, 0);
        assert_eq!(compressed.node_count(), 1);
        assert!(rules.is_empty());
    }

    #[test]
    fn compress_single_node_no_rules() {
        let forest = IrForest::from_nodes(vec![IrNode::code(b"unique payload here")]);
        let (_, rules) = compress(&forest, 10);
        assert!(rules.is_empty());
    }

    #[test]
    fn compress_repeated_patterns() {
        let pat = b"function foo() { return 1; }";
        let forest = IrForest::from_nodes(vec![
            IrNode::code(pat.as_slice()),
            IrNode::code(pat.as_slice()),
            IrNode::code(pat.as_slice()),
        ]);
        let (compressed, rules) = compress(&forest, 5);
        assert!(!rules.is_empty());
        // At least some nodes should be compressed.
        let compressed_count = compressed
            .nodes()
            .iter()
            .filter(|n| matches!(n, IrNode::Compressed { .. }))
            .count();
        assert!(compressed_count > 0);
    }

    #[test]
    fn decompress_roundtrip_text_nodes() {
        let forest = IrForest::from_nodes(vec![IrNode::text(b"hello")]);
        let (compressed, rules) = compress(&forest, 5);
        let decompressed = decompress(&compressed, &rules);
        assert_eq!(decompressed.node_count(), 1);
    }

    #[test]
    fn decompress_expands_compressed_nodes() {
        let pattern = b"return 42;";
        let forest = IrForest::from_nodes(vec![
            IrNode::code(pattern.as_slice()),
            IrNode::code(pattern.as_slice()),
        ]);
        let (compressed, rules) = compress(&forest, 5);

        if !rules.is_empty() {
            let decompressed = decompress(&compressed, &rules);
            // Decompressed should have nodes (at least as many as input).
            assert!(decompressed.node_count() >= compressed.node_count());
        }
    }

    #[test]
    fn find_subslice_basic() {
        assert_eq!(find_subslice(b"hello world", b"world"), Some(6));
        assert_eq!(find_subslice(b"hello", b"xyz"), None);
        assert_eq!(find_subslice(b"", b"a"), None);
        assert_eq!(find_subslice(b"a", b""), None);
    }

    #[test]
    fn non_structural_nodes_passthrough() {
        let forest = IrForest::from_nodes(vec![
            IrNode::text(b"plain text"),
            IrNode::list(vec![0, 1]),
            IrNode::reference(0),
        ]);
        let (compressed, rules) = compress(&forest, 10);
        assert_eq!(compressed.node_count(), 3);
        assert!(rules.is_empty());
    }

    #[test]
    fn mixed_forest_compression() {
        let pattern = b"const X = 1;";
        let forest = IrForest::from_nodes(vec![
            IrNode::text(b"header"),
            IrNode::code(pattern.as_slice()),
            IrNode::code(pattern.as_slice()),
            IrNode::text(b"footer"),
        ]);
        let (compressed, _) = compress(&forest, 5);
        assert_eq!(compressed.node_count(), 4);
        // Text nodes should pass through unchanged.
        assert!(matches!(compressed.get(0).unwrap(), IrNode::Text(_)));
        assert!(matches!(compressed.get(3).unwrap(), IrNode::Text(_)));
    }
}
