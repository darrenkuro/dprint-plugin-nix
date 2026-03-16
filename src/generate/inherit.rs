use dprint_core::formatting::{PrintItems, Signal};
use rnix::{SyntaxKind, SyntaxNode};

use super::gen_node;
use crate::config::NixConfig;

/// Generate IR for `inherit a b c;` or `inherit (source) a b c;`
pub fn gen_inherit(node: &SyntaxNode, config: &NixConfig) -> PrintItems {
    let mut p = PrintItems::new();
    let mut last_ws_had_newline = false;
    let mut has_items = false;

    p.push_str_runtime_width_computed("inherit");

    for child in node.children_with_tokens() {
        match child.kind() {
            SyntaxKind::TOKEN_WHITESPACE => {
                let text = child.as_token().unwrap().text();
                last_ws_had_newline = text.contains('\n');
            }
            SyntaxKind::TOKEN_COMMENT => {
                let text = child.as_token().unwrap().text().to_string();
                if !last_ws_had_newline && has_items {
                    p.extend(super::comment::gen_trailing_comment(&text));
                } else {
                    p.extend(super::comment::gen_comment(&text));
                }
                last_ws_had_newline = true;
            }
            SyntaxKind::TOKEN_INHERIT => {} // Already emitted
            SyntaxKind::TOKEN_SEMICOLON => {
                p.push_str_runtime_width_computed(";");
                last_ws_had_newline = false;
            }
            SyntaxKind::NODE_INHERIT_FROM => {
                p.push_signal(Signal::SpaceIfNotTrailing);
                if let Some(n) = child.as_node() {
                    p.extend(gen_inherit_from(n, config));
                }
                has_items = true;
                last_ws_had_newline = false;
            }
            _ => {
                p.push_signal(Signal::SpaceIfNotTrailing);
                if let Some(n) = child.as_node() {
                    p.extend(gen_node(n, config));
                } else if let Some(t) = child.as_token() {
                    p.push_string(t.text().to_string());
                }
                has_items = true;
                last_ws_had_newline = false;
            }
        }
    }

    p
}

/// Generate IR for the `(source)` part of inherit
fn gen_inherit_from(node: &SyntaxNode, config: &NixConfig) -> PrintItems {
    let mut p = PrintItems::new();
    p.push_str_runtime_width_computed("(");

    for child in node.children_with_tokens() {
        match child.kind() {
            SyntaxKind::TOKEN_L_PAREN | SyntaxKind::TOKEN_R_PAREN => {}
            SyntaxKind::TOKEN_WHITESPACE | SyntaxKind::TOKEN_COMMENT => {}
            _ => {
                if let Some(n) = child.as_node() {
                    p.extend(gen_node(n, config));
                } else if let Some(t) = child.as_token() {
                    p.push_string(t.text().to_string());
                }
            }
        }
    }

    p.push_str_runtime_width_computed(")");
    p
}
