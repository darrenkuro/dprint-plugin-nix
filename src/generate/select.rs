use dprint_core::formatting::PrintItems;
use rnix::{SyntaxKind, SyntaxNode};

use super::gen_node;
use crate::config::NixConfig;

/// Generate IR for attribute selection: `a.b`, `a.b.c`, `a.b or default`
pub fn gen_select(node: &SyntaxNode, config: &NixConfig) -> PrintItems {
    let mut p = PrintItems::new();
    let mut last_ws_had_newline = false;

    for child in node.children_with_tokens() {
        match child.kind() {
            SyntaxKind::TOKEN_WHITESPACE => {
                let text = child.as_token().unwrap().text();
                last_ws_had_newline = text.contains('\n');
            }
            SyntaxKind::TOKEN_COMMENT => {
                let text = child.as_token().unwrap().text().to_string();
                if !last_ws_had_newline {
                    p.extend(super::comment::gen_trailing_comment(&text));
                } else {
                    p.extend(super::comment::gen_comment(&text));
                }
                last_ws_had_newline = true;
            }
            SyntaxKind::TOKEN_DOT => {
                p.push_str_runtime_width_computed(".");
                last_ws_had_newline = false;
            }
            SyntaxKind::TOKEN_OR => {
                p.push_str_runtime_width_computed(" or ");
                last_ws_had_newline = false;
            }
            _ => {
                if let Some(n) = child.as_node() {
                    p.extend(gen_node(n, config));
                } else if let Some(t) = child.as_token() {
                    p.push_string(t.text().to_string());
                }
                last_ws_had_newline = false;
            }
        }
    }

    p
}

/// Generate IR for `a ? b` (has attribute)
pub fn gen_has_attr(node: &SyntaxNode, config: &NixConfig) -> PrintItems {
    let mut p = PrintItems::new();
    let mut last_ws_had_newline = false;

    for child in node.children_with_tokens() {
        match child.kind() {
            SyntaxKind::TOKEN_WHITESPACE => {
                let text = child.as_token().unwrap().text();
                last_ws_had_newline = text.contains('\n');
            }
            SyntaxKind::TOKEN_COMMENT => {
                let text = child.as_token().unwrap().text().to_string();
                if !last_ws_had_newline {
                    p.extend(super::comment::gen_trailing_comment(&text));
                } else {
                    p.extend(super::comment::gen_comment(&text));
                }
                last_ws_had_newline = true;
            }
            SyntaxKind::TOKEN_QUESTION => {
                p.push_str_runtime_width_computed(" ? ");
                last_ws_had_newline = false;
            }
            _ => {
                if let Some(n) = child.as_node() {
                    p.extend(gen_node(n, config));
                } else if let Some(t) = child.as_token() {
                    p.push_string(t.text().to_string());
                }
                last_ws_had_newline = false;
            }
        }
    }

    p
}

/// Generate IR for unary operations: `!a`, `-a`
pub fn gen_unary_op(node: &SyntaxNode, config: &NixConfig) -> PrintItems {
    let mut p = PrintItems::new();

    for child in node.children_with_tokens() {
        match child.kind() {
            SyntaxKind::TOKEN_WHITESPACE => {}
            SyntaxKind::TOKEN_COMMENT => {
                let text = child.as_token().unwrap().text().to_string();
                p.extend(super::comment::gen_trailing_comment(&text));
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
