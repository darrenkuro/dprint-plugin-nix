use dprint_core::formatting::{PrintItems, Signal};
use rnix::{SyntaxKind, SyntaxNode};

use super::gen_node;
use crate::config::NixConfig;

/// Generate IR for `with expr; body` and `assert expr; body`
pub fn gen_with_assert(node: &SyntaxNode, config: &NixConfig) -> PrintItems {
    let mut p = PrintItems::new();
    let mut phase = 0; // 0=keyword, 1=expr, 2=semi, 3=body
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
                    p.push_signal(Signal::NewLine);
                    p.extend(super::comment::gen_comment(&text));
                }
                last_ws_had_newline = true;
            }
            SyntaxKind::TOKEN_WITH | SyntaxKind::TOKEN_ASSERT => {
                p.push_string(child.as_token().unwrap().text().to_string());
                p.push_signal(Signal::SpaceIfNotTrailing);
                phase = 1;
            }
            SyntaxKind::TOKEN_SEMICOLON => {
                p.push_str_runtime_width_computed(";");
                phase = 2;
            }
            _ => {
                let inner = if let Some(n) = child.as_node() {
                    gen_node(n, config)
                } else if let Some(t) = child.as_token() {
                    let mut items = PrintItems::new();
                    items.push_string(t.text().to_string());
                    items
                } else {
                    continue;
                };

                match phase {
                    1 => {
                        p.extend(inner);
                    }
                    2 | 3 => {
                        p.push_signal(Signal::NewLine);
                        p.extend(inner);
                        phase = 3;
                    }
                    _ => {
                        p.extend(inner);
                    }
                }
                last_ws_had_newline = false;
            }
        }
    }

    p
}
