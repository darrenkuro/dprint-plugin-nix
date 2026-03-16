use dprint_core::formatting::{PrintItems, Signal};
use rnix::{SyntaxKind, SyntaxNode};

use super::gen_node;
use crate::config::NixConfig;

pub fn gen_if_else(node: &SyntaxNode, config: &NixConfig) -> PrintItems {
    let mut p = PrintItems::new();
    let mut phase = 0; // 0=if, 1=cond, 2=then, 3=then_body, 4=else, 5=else_body
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
            SyntaxKind::TOKEN_IF => {
                p.push_str_runtime_width_computed("if");
                p.push_signal(Signal::SpaceIfNotTrailing);
                phase = 1;
            }
            SyntaxKind::TOKEN_THEN => {
                p.push_signal(Signal::NewLine);
                p.push_str_runtime_width_computed("then");
                phase = 2;
            }
            SyntaxKind::TOKEN_ELSE => {
                p.push_signal(Signal::NewLine);
                p.push_str_runtime_width_computed("else");
                phase = 4;
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
                        phase = 2;
                    }
                    2 | 3 => {
                        p.push_signal(Signal::StartIndent);
                        p.push_signal(Signal::NewLine);
                        p.extend(inner);
                        p.push_signal(Signal::FinishIndent);
                        phase = 3;
                    }
                    4 | 5 => {
                        if child.kind() == SyntaxKind::NODE_IF_ELSE {
                            p.push_signal(Signal::SpaceIfNotTrailing);
                            p.extend(inner);
                        } else {
                            p.push_signal(Signal::StartIndent);
                            p.push_signal(Signal::NewLine);
                            p.extend(inner);
                            p.push_signal(Signal::FinishIndent);
                        }
                        phase = 5;
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
