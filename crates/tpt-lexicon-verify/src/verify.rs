//! Verification engine: structural invariant checking for IR forests.

use alloc::vec;
use alloc::vec::Vec;

use crate::error::{Error, Result, VerifyError};
use tpt_lexicon_ir::{CompressRule, IrForest, IrNode};

/// A report of verification results.
#[derive(Debug, Clone)]
pub struct VerificationReport {
    /// Whether the IR passed all checks.
    pub passed: bool,
    /// List of errors found (empty if passed).
    pub errors: Vec<VerifyError>,
}

impl VerificationReport {
    /// Create a passing report.
    #[inline]
    pub fn pass() -> Self {
        Self {
            passed: true,
            errors: vec![],
        }
    }

    /// Create a failing report with the given errors.
    #[inline]
    pub fn fail(errors: Vec<VerifyError>) -> Self {
        Self {
            passed: false,
            errors,
        }
    }
}

/// Trait for types that can be structurally verified.
pub trait Verify {
    /// Run all structural invariant checks and return a report.
    fn verify(&self) -> VerificationReport;
}

impl<'a> Verify for IrForest<'a> {
    fn verify(&self) -> VerificationReport {
        let mut errors = Vec::new();
        let node_count = self.node_count();

        for (i, node) in self.nodes().iter().enumerate() {
            match node {
                IrNode::Text(bytes) | IrNode::Code(bytes) | IrNode::Structured(bytes) => {
                    check_delimiters(bytes, i, &mut errors);
                }
                IrNode::List(indices) => {
                    for &idx in indices {
                        if idx >= node_count {
                            errors.push(VerifyError::InvalidReference {
                                node_index: i,
                                target_index: idx,
                                bound: node_count,
                            });
                        }
                    }
                }
                IrNode::Reference(idx) => {
                    if *idx >= node_count {
                        errors.push(VerifyError::InvalidReference {
                            node_index: i,
                            target_index: *idx,
                            bound: node_count,
                        });
                    }
                }
                IrNode::Compressed {
                    rule_index: _,
                    args,
                } => {
                    // Check that all args are valid references.
                    for &arg in args {
                        if arg >= node_count {
                            errors.push(VerifyError::InvalidReference {
                                node_index: i,
                                target_index: arg,
                                bound: node_count,
                            });
                        }
                    }
                }
            }
        }

        // Check for cycles in the reference graph.
        check_cycles(self, &mut errors);

        if errors.is_empty() {
            VerificationReport::pass()
        } else {
            VerificationReport::fail(errors)
        }
    }
}

/// Check for balanced delimiters (brackets, braces, parens) in a byte slice.
fn check_delimiters(bytes: &[u8], _node_index: usize, errors: &mut Vec<VerifyError>) {
    let mut stack: Vec<u8> = Vec::new();
    let mut first_error = true;

    for (offset, &b) in bytes.iter().enumerate() {
        if !first_error {
            break;
        }
        match b {
            b'(' | b'[' | b'{' => stack.push(b),
            b')' if stack.pop() != Some(b'(') => {
                errors.push(VerifyError::UnbalancedDelimiter {
                    offset,
                    delimiter: b')',
                });
                first_error = false;
            }
            b']' if stack.pop() != Some(b'[') => {
                errors.push(VerifyError::UnbalancedDelimiter {
                    offset,
                    delimiter: b']',
                });
                first_error = false;
            }
            b'}' if stack.pop() != Some(b'{') => {
                errors.push(VerifyError::UnbalancedDelimiter {
                    offset,
                    delimiter: b'}',
                });
                first_error = false;
            }
            _ => {}
        }
    }

    // Check for unclosed openers at end of input.
    if first_error && !stack.is_empty() {
        let last = *stack.last().unwrap();
        errors.push(VerifyError::UnbalancedDelimiter {
            offset: bytes.len(),
            delimiter: last,
        });
    }
}

/// Check for cycles in the reference graph using DFS.
fn check_cycles<'a>(forest: &IrForest<'a>, errors: &mut Vec<VerifyError>) {
    let node_count = forest.node_count();
    let mut visited = vec![false; node_count];
    let mut in_stack = vec![false; node_count];

    fn dfs(
        node_index: usize,
        forest: &IrForest<'_>,
        visited: &mut [bool],
        in_stack: &mut [bool],
        errors: &mut Vec<VerifyError>,
    ) {
        if visited[node_index] {
            if in_stack[node_index] {
                errors.push(VerifyError::CycleDetected { node_index });
            }
            return;
        }

        visited[node_index] = true;
        in_stack[node_index] = true;

        match forest.get_unchecked(node_index) {
            IrNode::List(indices) => {
                for &idx in indices {
                    if idx < visited.len() {
                        dfs(idx, forest, visited, in_stack, errors);
                    }
                }
            }
            IrNode::Reference(idx) => {
                if *idx < visited.len() {
                    dfs(*idx, forest, visited, in_stack, errors);
                }
            }
            IrNode::Compressed { args, .. } => {
                for &arg in args {
                    if arg < visited.len() {
                        dfs(arg, forest, visited, in_stack, errors);
                    }
                }
            }
            _ => {}
        }

        in_stack[node_index] = false;
    }

    for i in 0..node_count {
        if !visited[i] {
            dfs(i, forest, &mut visited, &mut in_stack, errors);
        }
    }
}

/// Verify an IR forest and return a Result.
///
/// Returns `Ok(report)` with `report.passed == true` if all checks pass.
/// Returns `Err(Error)` if any check fails.
pub fn verify_ir(forest: &IrForest<'_>) -> Result<VerificationReport> {
    let report = forest.verify();
    if report.passed {
        Ok(report)
    } else {
        let first = report.errors[0].clone();
        Err(Error::VerificationFailed(first))
    }
}

/// Verify an IR forest against a rule table.
///
/// Runs all structural invariant checks from [`Verify::verify`] and
/// additionally validates every [`IrNode::Compressed`] node:
///
/// - `rule_index` must be within bounds of `rules`
/// - `args.len()` must equal `rules[rule_index].arity`
///
/// Returns `Err` on the first violation found.
pub fn verify_with_rules<'a>(
    forest: &IrForest<'a>,
    rules: &[CompressRule<'_>],
) -> Result<VerificationReport> {
    let base = forest.verify();
    let mut errors = base.errors;

    for (i, node) in forest.nodes().iter().enumerate() {
        if let IrNode::Compressed { rule_index, args } = node {
            if *rule_index >= rules.len() {
                errors.push(VerifyError::InvalidRuleReference {
                    node_index: i,
                    rule_index: *rule_index,
                });
            } else {
                let expected = rules[*rule_index].arity;
                let actual = args.len();
                if actual != expected {
                    errors.push(VerifyError::ArityMismatch {
                        node_index: i,
                        expected,
                        actual,
                    });
                }
            }
        }
    }

    if errors.is_empty() {
        Ok(VerificationReport::pass())
    } else {
        let first = errors[0].clone();
        Err(Error::VerificationFailed(first))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::VERSION;

    #[test]
    fn empty_forest_passes() {
        let forest = IrForest::new();
        let report = forest.verify();
        assert!(report.passed);
        assert!(report.errors.is_empty());
    }

    #[test]
    fn text_nodes_with_balanced_delimiters() {
        let forest = IrForest::from_nodes(vec![IrNode::text(b"hello (world) [test] {ok}")]);
        let report = forest.verify();
        assert!(report.passed);
    }

    #[test]
    fn text_nodes_with_unbalanced_paren() {
        let forest = IrForest::from_nodes(vec![IrNode::text(b"hello (world")]);
        let report = forest.verify();
        assert!(!report.passed);
        assert!(report
            .errors
            .iter()
            .any(|e| matches!(e, VerifyError::UnbalancedDelimiter { .. })));
    }

    #[test]
    fn text_nodes_with_wrong_closing() {
        let forest = IrForest::from_nodes(vec![IrNode::text(b"(hello]")]);
        let report = forest.verify();
        assert!(!report.passed);
    }

    #[test]
    fn valid_references() {
        let forest = IrForest::from_nodes(vec![
            IrNode::text(b"node0"),
            IrNode::Reference(0),
            IrNode::list(vec![0, 1]),
        ]);
        let report = forest.verify();
        assert!(report.passed);
    }

    #[test]
    fn invalid_reference_out_of_bounds() {
        let forest = IrForest::from_nodes(vec![IrNode::Reference(5)]);
        let report = forest.verify();
        assert!(!report.passed);
        assert!(report
            .errors
            .iter()
            .any(|e| matches!(e, VerifyError::InvalidReference { .. })));
    }

    #[test]
    fn invalid_list_reference() {
        let forest = IrForest::from_nodes(vec![IrNode::list(vec![0, 10])]);
        let report = forest.verify();
        assert!(!report.passed);
    }

    #[test]
    fn cycle_detection() {
        // Node 0 references node 1, node 1 references node 0.
        let forest = IrForest::from_nodes(vec![IrNode::Reference(1), IrNode::Reference(0)]);
        let report = forest.verify();
        assert!(!report.passed);
        assert!(report
            .errors
            .iter()
            .any(|e| matches!(e, VerifyError::CycleDetected { .. })));
    }

    #[test]
    fn no_false_positive_self_reference_to_valid_node() {
        let forest = IrForest::from_nodes(vec![
            IrNode::text(b"a"),
            IrNode::text(b"b"),
            IrNode::Reference(1),
        ]);
        let report = forest.verify();
        assert!(report.passed);
    }

    #[test]
    fn verify_ir_passes() {
        let forest = IrForest::from_nodes(vec![IrNode::text(b"ok")]);
        let result = verify_ir(&forest);
        assert!(result.is_ok());
    }

    #[test]
    fn verify_ir_fails() {
        let forest = IrForest::from_nodes(vec![IrNode::Reference(99)]);
        let result = verify_ir(&forest);
        assert!(result.is_err());
    }

    #[test]
    fn multiple_errors_collected() {
        let forest = IrForest::from_nodes(vec![
            IrNode::text(b"(]"),
            IrNode::Reference(10),
            IrNode::list(vec![20]),
        ]);
        let report = forest.verify();
        assert!(!report.passed);
        assert!(report.errors.len() >= 2);
    }

    #[test]
    fn balanced_nested_delimiters() {
        let forest = IrForest::from_nodes(vec![IrNode::text(b"{[()]}")]);
        let report = forest.verify();
        assert!(report.passed);
    }

    #[test]
    fn code_nodes_verified() {
        let forest = IrForest::from_nodes(vec![IrNode::code(b"fn foo() { }")]);
        let report = forest.verify();
        assert!(report.passed);
    }

    #[test]
    fn structured_nodes_verified() {
        let forest = IrForest::from_nodes(vec![IrNode::structured(b"[1, 2, 3]")]);
        let report = forest.verify();
        assert!(report.passed);
    }

    #[test]
    fn compressed_node_with_valid_args() {
        let forest = IrForest::from_nodes(vec![
            IrNode::text(b"a"),
            IrNode::Compressed {
                rule_index: 0,
                args: vec![0],
            },
        ]);
        let report = forest.verify();
        assert!(report.passed);
    }

    #[test]
    fn compressed_node_with_invalid_arg() {
        let forest = IrForest::from_nodes(vec![IrNode::Compressed {
            rule_index: 0,
            args: vec![99],
        }]);
        let report = forest.verify();
        assert!(!report.passed);
    }

    #[test]
    fn version_is_nonempty() {
        assert!(!VERSION.is_empty());
    }

    // verify_with_rules tests
    #[test]
    fn verify_with_rules_valid() {
        use tpt_lexicon_ir::{CompressRule, TemplateToken};
        let rule = CompressRule::new(vec![TemplateToken::Literal(b"hello")], 0);
        let forest = IrForest::from_nodes(vec![
            IrNode::text(b"node0"),
            IrNode::Compressed {
                rule_index: 0,
                args: vec![],
            },
        ]);
        assert!(super::verify_with_rules(&forest, &[rule]).is_ok());
    }

    #[test]
    fn verify_with_rules_invalid_rule_index() {
        use tpt_lexicon_ir::{CompressRule, TemplateToken};
        let rule = CompressRule::new(vec![TemplateToken::Literal(b"x")], 0);
        let forest = IrForest::from_nodes(vec![IrNode::Compressed {
            rule_index: 5, // out of bounds — only rule 0 exists
            args: vec![],
        }]);
        let result = super::verify_with_rules(&forest, &[rule]);
        assert!(result.is_err());
    }

    #[test]
    fn verify_with_rules_arity_mismatch() {
        use tpt_lexicon_ir::{CompressRule, IrNode, TemplateToken};
        // Rule has arity 1 but the node provides 0 args.
        let rule = CompressRule::new(vec![TemplateToken::Placeholder(0)], 1);
        let forest = IrForest::from_nodes(vec![IrNode::Compressed {
            rule_index: 0,
            args: vec![], // should have 1 arg
        }]);
        let result = super::verify_with_rules(&forest, &[rule]);
        assert!(result.is_err());
    }
}
