pub mod apply;
pub mod attr_set;
pub mod bin_op;
pub mod comment;
pub mod helpers;
pub mod if_else;
pub mod inherit;
pub mod key_value;
pub mod lambda;
pub mod let_in;
pub mod list;
pub mod paren;
pub mod select;
pub mod string;
pub mod with_assert;

use dprint_core::formatting::{PrintItems, Signal};
use rnix::{SyntaxKind, SyntaxNode};

use crate::config::NixConfig;

/// Main dispatch: generate PrintItems IR for any Nix syntax node
pub fn gen_node(node: &SyntaxNode, config: &NixConfig) -> PrintItems {
    match node.kind() {
        // Root
        SyntaxKind::NODE_ROOT => gen_root(node, config),

        // Containers
        SyntaxKind::NODE_ATTR_SET => attr_set::gen_attr_set(node, config),
        SyntaxKind::NODE_LIST => list::gen_list(node, config),

        // Bindings
        SyntaxKind::NODE_ATTRPATH_VALUE => key_value::gen_key_value(node, config),
        SyntaxKind::NODE_ATTRPATH => key_value::gen_attrpath(node, config),
        SyntaxKind::NODE_INHERIT => inherit::gen_inherit(node, config),
        SyntaxKind::NODE_INHERIT_FROM => {
            // Handled inside inherit.rs
            gen_children_flat(node, config)
        }

        // Control flow
        SyntaxKind::NODE_LET_IN => let_in::gen_let_in(node, config),
        SyntaxKind::NODE_IF_ELSE => if_else::gen_if_else(node, config),
        SyntaxKind::NODE_WITH => with_assert::gen_with_assert(node, config),
        SyntaxKind::NODE_ASSERT => with_assert::gen_with_assert(node, config),

        // Functions
        SyntaxKind::NODE_LAMBDA => lambda::gen_lambda(node, config),
        SyntaxKind::NODE_PATTERN => lambda::gen_pattern(node, config),
        SyntaxKind::NODE_PAT_ENTRY => lambda::gen_pat_entry(node, config),
        SyntaxKind::NODE_PAT_BIND => gen_children_spaced(node, config),
        SyntaxKind::NODE_IDENT_PARAM => gen_children_flat(node, config),

        // Expressions
        SyntaxKind::NODE_APPLY => apply::gen_apply(node, config),
        SyntaxKind::NODE_BIN_OP => bin_op::gen_bin_op(node, config),
        SyntaxKind::NODE_UNARY_OP => select::gen_unary_op(node, config),
        SyntaxKind::NODE_PAREN => paren::gen_paren(node, config),
        SyntaxKind::NODE_SELECT => select::gen_select(node, config),
        SyntaxKind::NODE_HAS_ATTR => select::gen_has_attr(node, config),

        // Strings
        SyntaxKind::NODE_STRING => string::gen_string(node, config),
        SyntaxKind::NODE_INTERPOL => paren::gen_interpol(node, config),

        // Literals and simple nodes — pass through
        SyntaxKind::NODE_LITERAL
        | SyntaxKind::NODE_IDENT
        | SyntaxKind::NODE_PATH
        | SyntaxKind::NODE_DYNAMIC => gen_children_flat(node, config),

        // Legacy / rare
        SyntaxKind::NODE_LEGACY_LET => gen_children_flat(node, config),

        // Fallback: unknown node types — preserve original text
        _ => {
            let mut p = PrintItems::new();
            p.push_string(node.text().to_string());
            p
        }
    }
}

/// Generate IR for root node (entire file)
fn gen_root(node: &SyntaxNode, config: &NixConfig) -> PrintItems {
    let mut p = PrintItems::new();
    let mut first = true;
    let mut blank_line_pending = false;
    let mut last_ws_had_newline = false;

    for child in node.children_with_tokens() {
        match child.kind() {
            SyntaxKind::TOKEN_WHITESPACE => {
                let text = child.as_token().unwrap().text();
                let newlines = text.chars().filter(|c| *c == '\n').count();
                last_ws_had_newline = newlines > 0;
                if newlines > 1 && !first {
                    blank_line_pending = true;
                }
            }
            SyntaxKind::TOKEN_COMMENT => {
                let text = child.as_token().unwrap().text().to_string();

                if !last_ws_had_newline && !first {
                    // Trailing comment — attach to output (same line as preceding code)
                    p.extend(comment::gen_trailing_comment(&text));
                } else {
                    // Standalone/leading comment
                    if !first {
                        if blank_line_pending {
                            p.push_signal(Signal::NewLine);
                            blank_line_pending = false;
                        }
                        p.push_signal(Signal::NewLine);
                    }
                    p.extend(comment::gen_comment(&text));
                }
                first = false;
                last_ws_had_newline = true;
            }
            _ => {
                if let Some(child_node) = child.as_node() {
                    if !first {
                        if blank_line_pending {
                            p.push_signal(Signal::NewLine);
                            blank_line_pending = false;
                        }
                        p.push_signal(Signal::NewLine);
                    }
                    p.extend(gen_node(child_node, config));
                    first = false;
                } else if let Some(token) = child.as_token() {
                    if !helpers::is_trivial_token(child.kind()) {
                        if !first {
                            p.push_signal(Signal::NewLine);
                        }
                        p.push_string(token.text().to_string());
                        first = false;
                    }
                }
                last_ws_had_newline = false;
            }
        }
    }

    // Ensure file ends with newline
    p.push_signal(Signal::NewLine);
    p
}

/// Flatten all children of a node into print items (no extra formatting)
fn gen_children_flat(node: &SyntaxNode, config: &NixConfig) -> PrintItems {
    let mut p = PrintItems::new();

    for child in node.children_with_tokens() {
        match child.kind() {
            SyntaxKind::TOKEN_WHITESPACE => {}
            SyntaxKind::TOKEN_COMMENT => {
                let text = child.as_token().unwrap().text().to_string();
                p.extend(comment::gen_trailing_comment(&text));
            }
            _ => {
                if let Some(n) = child.as_node() {
                    p.extend(gen_node(n, config));
                } else if let Some(t) = child.as_token() {
                    p.push_string(t.text().to_string());
                }
            }
        }
    }

    p
}

/// Children with spaces between them
fn gen_children_spaced(node: &SyntaxNode, config: &NixConfig) -> PrintItems {
    let mut p = PrintItems::new();
    let mut first = true;

    for child in node.children_with_tokens() {
        match child.kind() {
            SyntaxKind::TOKEN_WHITESPACE => {}
            SyntaxKind::TOKEN_COMMENT => {
                let text = child.as_token().unwrap().text().to_string();
                p.extend(comment::gen_trailing_comment(&text));
            }
            _ => {
                if !first {
                    p.push_signal(Signal::SpaceIfNotTrailing);
                }

                if let Some(n) = child.as_node() {
                    p.extend(gen_node(n, config));
                } else if let Some(t) = child.as_token() {
                    p.push_string(t.text().to_string());
                }

                first = false;
            }
        }
    }

    p
}

/// Generate trivia (comments/blank lines) before a semantic node
pub fn gen_trivia_before(_node: &SyntaxNode) -> PrintItems {
    let p = PrintItems::new();
    // This is a placeholder — proper trivia handling will be refined
    p
}
