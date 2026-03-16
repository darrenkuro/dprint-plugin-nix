use dprint_core::formatting::PrintItems;
use rnix::{SyntaxKind, SyntaxNode};

use super::gen_node;
use crate::config::NixConfig;

/// Generate IR for `name = value;` (NODE_ATTRPATH_VALUE)
pub fn gen_key_value(node: &SyntaxNode, config: &NixConfig) -> PrintItems {
    let mut p = PrintItems::new();
    let mut phase = 0; // 0=key, 1=seen_eq, 2=value, 3=semi
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
            _ => {
                match phase {
                    0 => {
                        if child.kind() == SyntaxKind::TOKEN_ASSIGN {
                            p.push_str_runtime_width_computed(" = ");
                            phase = 1;
                        } else if let Some(n) = child.as_node() {
                            p.extend(gen_node(n, config));
                        } else if let Some(t) = child.as_token() {
                            p.push_string(t.text().to_string());
                        }
                    }
                    1 => {
                        let value_items = if let Some(n) = child.as_node() {
                            gen_node(n, config)
                        } else if let Some(t) = child.as_token() {
                            let mut v = PrintItems::new();
                            v.push_string(t.text().to_string());
                            v
                        } else {
                            PrintItems::new()
                        };
                        p.extend(value_items);
                        phase = 2;
                    }
                    2 => {
                        if child.kind() == SyntaxKind::TOKEN_SEMICOLON {
                            p.push_str_runtime_width_computed(";");
                        }
                        phase = 3;
                    }
                    _ => {}
                }
                last_ws_had_newline = false;
            }
        }
    }

    p
}

/// Generate IR for attribute path (NODE_ATTRPATH): `a.b.c`
pub fn gen_attrpath(node: &SyntaxNode, config: &NixConfig) -> PrintItems {
    let mut p = PrintItems::new();

    for child in node.children_with_tokens() {
        if super::helpers::is_trivial_token(child.kind()) {
            continue;
        }

        if child.kind() == SyntaxKind::TOKEN_DOT {
            p.push_str_runtime_width_computed(".");
        } else if let Some(n) = child.as_node() {
            p.extend(gen_node(n, config));
        } else if let Some(t) = child.as_token() {
            p.push_string(t.text().to_string());
        }
    }

    p
}
