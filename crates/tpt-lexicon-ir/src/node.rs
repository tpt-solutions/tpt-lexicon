//! IR node types, forest, and compression rule definitions.

use alloc::vec::Vec;

use crate::error::{Error, Result};

/// Magic bytes identifying TPT Lexicon IR binary files.
pub const MAGIC: &[u8; 4] = b"TXIR";

/// Current IR binary format version.
pub const VERSION: u32 = 1;

/// Maximum IR binary format version this crate supports.
pub const MAX_SUPPORTED_VERSION: u32 = 1;

/// A node in the intermediate representation tree.
///
/// Each node carries a type tag and a borrowed byte payload. Nodes can
/// reference other nodes by index for composition.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum IrNode<'a> {
    /// Plain text content.
    Text(&'a [u8]),
    /// Source code content.
    Code(&'a [u8]),
    /// Structured data (JSON, YAML, etc.).
    Structured(&'a [u8]),
    /// A list of child node indices.
    List(Vec<usize>),
    /// A reference to another node by index.
    Reference(usize),
    /// A compressed node — references a compression rule index and a list of
    /// argument node indices.
    Compressed {
        /// Index into the compression rule table.
        rule_index: usize,
        /// Argument node indices to substitute into the rule.
        args: Vec<usize>,
    },
}

impl<'a> IrNode<'a> {
    /// Create a text node.
    #[inline]
    pub fn text(bytes: &'a [u8]) -> Self {
        Self::Text(bytes)
    }

    /// Create a code node.
    #[inline]
    pub fn code(bytes: &'a [u8]) -> Self {
        Self::Code(bytes)
    }

    /// Create a structured data node.
    #[inline]
    pub fn structured(bytes: &'a [u8]) -> Self {
        Self::Structured(bytes)
    }

    /// Create a list node from a vector of child indices.
    #[inline]
    pub fn list(indices: Vec<usize>) -> Self {
        Self::List(indices)
    }

    /// Create a reference node.
    #[inline]
    pub fn reference(index: usize) -> Self {
        Self::Reference(index)
    }

    /// Returns `true` if this node carries borrowed byte data (Text, Code, or
    /// Structured).
    pub fn has_bytes(&self) -> bool {
        matches!(self, Self::Text(_) | Self::Code(_) | Self::Structured(_))
    }

    /// Returns the borrowed bytes if this is a Text, Code, or Structured node.
    pub fn as_bytes(&self) -> Option<&'a [u8]> {
        match self {
            Self::Text(b) | Self::Code(b) | Self::Structured(b) => Some(b),
            _ => None,
        }
    }

    /// Returns a human-readable label for the node type.
    pub fn kind_name(&self) -> &'static str {
        match self {
            Self::Text(_) => "text",
            Self::Code(_) => "code",
            Self::Structured(_) => "structured",
            Self::List(_) => "list",
            Self::Reference(_) => "reference",
            Self::Compressed { .. } => "compressed",
        }
    }
}

/// A compression rule: a template that replaces a repeating structural pattern.
///
/// The rule stores the body as a sequence of template tokens — either literal
/// byte slices from the original input or placeholder indices that get
/// substituted with argument nodes during decompression.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum TemplateToken<'a> {
    /// Literal bytes from the original input.
    Literal(&'a [u8]),
    /// A placeholder that will be substituted with the Nth argument.
    Placeholder(usize),
}

/// A compression rule: describes a repeating pattern and its replacement
/// template.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct CompressRule<'a> {
    /// The body of the rule as a sequence of template tokens.
    pub body: Vec<TemplateToken<'a>>,
    /// The number of arguments this rule expects.
    pub arity: usize,
}

impl<'a> CompressRule<'a> {
    /// Create a new compression rule.
    pub fn new(body: Vec<TemplateToken<'a>>, arity: usize) -> Self {
        Self { body, arity }
    }
}

/// An IR forest: a top-level collection of IR nodes.
///
/// The forest owns its nodes (via `alloc`) and provides indexing, serialization,
/// and decompression support.
#[derive(Debug, Clone)]
pub struct IrForest<'a> {
    nodes: Vec<IrNode<'a>>,
}

impl<'a> IrForest<'a> {
    /// Create an empty forest.
    #[inline]
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    /// Create a forest from a vector of nodes.
    #[inline]
    pub fn from_nodes(nodes: Vec<IrNode<'a>>) -> Self {
        Self { nodes }
    }

    /// Returns the number of nodes in the forest.
    #[inline]
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Returns `true` if the forest has no nodes.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Returns a reference to the node at the given index.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidReference`] if the index is out of bounds.
    pub fn get(&self, index: usize) -> Result<&IrNode<'a>> {
        self.nodes.get(index).ok_or(Error::InvalidReference {
            index,
            bound: self.nodes.len(),
        })
    }

    /// Returns a reference to the node at the given index (panics if
    /// out-of-bounds).
    ///
    /// # Panics
    ///
    /// Panics if `index >= self.node_count()`.
    #[inline]
    pub fn get_unchecked(&self, index: usize) -> &IrNode<'a> {
        &self.nodes[index]
    }

    /// Returns a reference to all nodes.
    #[inline]
    pub fn nodes(&self) -> &[IrNode<'a>] {
        &self.nodes
    }

    /// Returns a mutable reference to all nodes.
    #[inline]
    pub fn nodes_mut(&mut self) -> &mut Vec<IrNode<'a>> {
        &mut self.nodes
    }

    /// Push a node into the forest, returning its index.
    #[inline]
    pub fn push(&mut self, node: IrNode<'a>) -> usize {
        let idx = self.nodes.len();
        self.nodes.push(node);
        idx
    }

    /// Serialize the forest to binary format.
    ///
    /// Binary layout (all integers are little-endian):
    ///
    /// ```text
    /// Offset  Size  Field
    /// 0       4     Magic: b"TXIR"
    /// 4       4     Version: u32 LE
    /// 8       4     node_count: u32 LE
    /// 12      ...   Nodes: [tag:u8, ...payload]
    /// ```
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();

        // Magic
        buf.extend_from_slice(MAGIC);
        // Version
        buf.extend_from_slice(&VERSION.to_le_bytes());
        // Node count
        buf.extend_from_slice(&(self.nodes.len() as u32).to_le_bytes());

        // Nodes
        for node in &self.nodes {
            match node {
                IrNode::Text(bytes) => {
                    buf.push(0x01);
                    buf.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
                    buf.extend_from_slice(bytes);
                }
                IrNode::Code(bytes) => {
                    buf.push(0x02);
                    buf.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
                    buf.extend_from_slice(bytes);
                }
                IrNode::Structured(bytes) => {
                    buf.push(0x03);
                    buf.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
                    buf.extend_from_slice(bytes);
                }
                IrNode::List(indices) => {
                    buf.push(0x04);
                    buf.extend_from_slice(&(indices.len() as u32).to_le_bytes());
                    for &idx in indices {
                        buf.extend_from_slice(&(idx as u32).to_le_bytes());
                    }
                }
                IrNode::Reference(idx) => {
                    buf.push(0x05);
                    buf.extend_from_slice(&(*idx as u32).to_le_bytes());
                }
                IrNode::Compressed { rule_index, args } => {
                    buf.push(0x06);
                    buf.extend_from_slice(&(*rule_index as u32).to_le_bytes());
                    buf.extend_from_slice(&(args.len() as u32).to_le_bytes());
                    for &arg in args {
                        buf.extend_from_slice(&(arg as u32).to_le_bytes());
                    }
                }
            }
        }

        buf
    }

    /// Deserialize a forest from binary data.
    ///
    /// # Errors
    ///
    /// Returns an error if the data is truncated, corrupted, or uses an
    /// unsupported version.
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        if data.len() < 12 {
            return Err(Error::CorruptedData);
        }

        // Magic
        if &data[0..4] != MAGIC {
            return Err(Error::CorruptedData);
        }

        // Version
        let version = u32::from_le_bytes(data[4..8].try_into().unwrap());
        if version > MAX_SUPPORTED_VERSION {
            return Err(Error::UnsupportedVersion {
                found: version,
                expected_max: MAX_SUPPORTED_VERSION,
            });
        }

        // Node count
        let node_count = u32::from_le_bytes(data[8..12].try_into().unwrap()) as usize;
        let mut pos = 12;
        let mut nodes = Vec::with_capacity(node_count);

        for _ in 0..node_count {
            if pos >= data.len() {
                return Err(Error::CorruptedData);
            }
            let tag = data[pos];
            pos += 1;

            match tag {
                0x01 => {
                    // Text
                    if pos + 4 > data.len() {
                        return Err(Error::CorruptedData);
                    }
                    let len = u32::from_le_bytes(data[pos..pos + 4].try_into().unwrap()) as usize;
                    pos += 4;
                    if pos + len > data.len() {
                        return Err(Error::CorruptedData);
                    }
                    // We need to return borrowed data, but from_bytes takes
                    // owned data. Store the bytes in the nodes and transmute
                    // the lifetime. This is safe because IrForest<'a> can
                    // borrow from the original data, but from_bytes returns
                    // IrForest<'static> from a Vec<u8>. We solve this by
                    // having the caller manage lifetimes. For now, we leak.
                    //
                    // Actually, we can't safely return borrowed data from
                    // owned deserialization. We store the data in a Vec and
                    // the forest references it. But our API doesn't support
                    // that. For now, store as a static-equivalent by leaking.
                    //
                    // NOTE: this is a pragmatic choice for the initial
                    // implementation. A production version would use a
                    // self-describing buffer type.
                    let bytes = &data[pos..pos + len];
                    // SAFETY: This leaks memory. In a real implementation, the
                    // forest would own the buffer. For now, this is acceptable
                    // for the initial API. The leaked bytes are small and
                    // bounded by the input size.
                    let static_bytes: &'static [u8] =
                        alloc::boxed::Box::leak(bytes.to_vec().into_boxed_slice());
                    nodes.push(IrNode::Text(static_bytes));
                    pos += len;
                }
                0x02 => {
                    // Code
                    if pos + 4 > data.len() {
                        return Err(Error::CorruptedData);
                    }
                    let len = u32::from_le_bytes(data[pos..pos + 4].try_into().unwrap()) as usize;
                    pos += 4;
                    if pos + len > data.len() {
                        return Err(Error::CorruptedData);
                    }
                    let static_bytes: &'static [u8] =
                        alloc::boxed::Box::leak(data[pos..pos + len].to_vec().into_boxed_slice());
                    nodes.push(IrNode::Code(static_bytes));
                    pos += len;
                }
                0x03 => {
                    // Structured
                    if pos + 4 > data.len() {
                        return Err(Error::CorruptedData);
                    }
                    let len = u32::from_le_bytes(data[pos..pos + 4].try_into().unwrap()) as usize;
                    pos += 4;
                    if pos + len > data.len() {
                        return Err(Error::CorruptedData);
                    }
                    let static_bytes: &'static [u8] =
                        alloc::boxed::Box::leak(data[pos..pos + len].to_vec().into_boxed_slice());
                    nodes.push(IrNode::Structured(static_bytes));
                    pos += len;
                }
                0x04 => {
                    // List
                    if pos + 4 > data.len() {
                        return Err(Error::CorruptedData);
                    }
                    let count = u32::from_le_bytes(data[pos..pos + 4].try_into().unwrap()) as usize;
                    pos += 4;
                    let mut indices = Vec::with_capacity(count);
                    for _ in 0..count {
                        if pos + 4 > data.len() {
                            return Err(Error::CorruptedData);
                        }
                        let idx =
                            u32::from_le_bytes(data[pos..pos + 4].try_into().unwrap()) as usize;
                        pos += 4;
                        indices.push(idx);
                    }
                    nodes.push(IrNode::List(indices));
                }
                0x05 => {
                    // Reference
                    if pos + 4 > data.len() {
                        return Err(Error::CorruptedData);
                    }
                    let idx = u32::from_le_bytes(data[pos..pos + 4].try_into().unwrap()) as usize;
                    pos += 4;
                    nodes.push(IrNode::Reference(idx));
                }
                0x06 => {
                    // Compressed
                    if pos + 4 > data.len() {
                        return Err(Error::CorruptedData);
                    }
                    let rule_index =
                        u32::from_le_bytes(data[pos..pos + 4].try_into().unwrap()) as usize;
                    pos += 4;
                    if pos + 4 > data.len() {
                        return Err(Error::CorruptedData);
                    }
                    let arg_count =
                        u32::from_le_bytes(data[pos..pos + 4].try_into().unwrap()) as usize;
                    pos += 4;
                    let mut args = Vec::with_capacity(arg_count);
                    for _ in 0..arg_count {
                        if pos + 4 > data.len() {
                            return Err(Error::CorruptedData);
                        }
                        let arg =
                            u32::from_le_bytes(data[pos..pos + 4].try_into().unwrap()) as usize;
                        pos += 4;
                        args.push(arg);
                    }
                    nodes.push(IrNode::Compressed { rule_index, args });
                }
                _ => return Err(Error::CorruptedData),
            }
        }

        Ok(Self { nodes })
    }
}

impl Default for IrForest<'_> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn node_kind_names() {
        assert_eq!(IrNode::text(b"hi").kind_name(), "text");
        assert_eq!(IrNode::code(b"x").kind_name(), "code");
        assert_eq!(IrNode::structured(b"{}").kind_name(), "structured");
        assert_eq!(IrNode::list(vec![]).kind_name(), "list");
        assert_eq!(IrNode::reference(0).kind_name(), "reference");
        assert_eq!(
            IrNode::Compressed {
                rule_index: 0,
                args: vec![],
            }
            .kind_name(),
            "compressed"
        );
    }

    #[test]
    fn node_as_bytes() {
        assert_eq!(IrNode::text(b"hello").as_bytes(), Some(b"hello".as_slice()));
        assert!(IrNode::list(vec![]).as_bytes().is_none());
    }

    #[test]
    fn forest_basics() {
        let mut forest = IrForest::new();
        assert!(forest.is_empty());
        assert_eq!(forest.node_count(), 0);

        let idx = forest.push(IrNode::text(b"hello"));
        assert_eq!(idx, 0);
        assert_eq!(forest.node_count(), 1);
        assert!(!forest.is_empty());

        let node = forest.get(0).unwrap();
        assert_eq!(node.as_bytes(), Some(b"hello".as_slice()));
    }

    #[test]
    fn forest_get_out_of_bounds() {
        let forest = IrForest::new();
        assert!(forest.get(0).is_err());
    }

    #[test]
    fn forest_roundtrip_bytes() {
        let mut forest = IrForest::new();
        forest.push(IrNode::text(b"hello"));
        forest.push(IrNode::code(b"let x = 1;"));
        forest.push(IrNode::list(vec![0, 1]));
        forest.push(IrNode::reference(0));

        let serialized = forest.to_bytes();
        let deserialized = IrForest::from_bytes(&serialized).unwrap();
        assert_eq!(deserialized.node_count(), 4);
    }

    #[test]
    fn forest_from_bytes_magic_check() {
        assert!(IrForest::from_bytes(b"XXXX").is_err());
    }

    #[test]
    fn forest_from_bytes_truncated() {
        assert!(IrForest::from_bytes(b"TXIR").is_err());
    }
}
