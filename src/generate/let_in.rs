use dprint_core::formatting::{PrintItems, Signal};
use rnix::{SyntaxKind, SyntaxNode};

use super::gen_node;
use crate::config::NixConfig;

pub fn gen_let_in(node: &SyntaxNode, config: &NixConfig) -> PrintItems {
    let mut p = PrintItems::new();
    let mut phase = 0; // 0=let, 1=bindings, 2=in, 3=body
    let mut blank_line_pending = false;
    let mut last_ws_had_newline = false;
    let mut has_bindings = false;

    for child in node.children_with_tokens() {
        match child.kind() {
            SyntaxKind::TOKEN_WHITESPACE => {
                let text = child.as_token().unwrap().text();
                let newlines = text.chars().filter(|c| *c == '\n').count();
                last_ws_had_newline = newlines > 0;
                if newlines > 1 && phase == 1 {
                    blank_line_pending = true;
                }
            }
            SyntaxKind::TOKEN_COMMENT => {
                let text = child.as_token().unwrap().text().to_string();
                match phase {
                    1 => {
                        if !last_ws_had_newline && has_bindings {
                            // Trailing comment on same line as binding
                            p.extend(super::comment::gen_trailing_comment(&text));
                        } else {
                            // Standalone comment between bindings
                            if blank_line_pending {
                                p.push_signal(Signal::NewLine);
                                blank_line_pending = false;
                            }
                            p.push_signal(Signal::NewLine);
                            p.extend(super::comment::gen_comment(&text));
                        }
                        has_bindings = true;
                    }
                    2 | 3 => {
                        // Comment after `in`
                        if !last_ws_had_newline {
                            p.extend(super::comment::gen_trailing_comment(&text));
                        } else {
                            p.push_signal(Signal::NewLine);
                            p.extend(super::comment::gen_comment(&text));
                        }
                    }
                    _ => {
                        p.extend(super::comment::gen_comment(&text));
                    }
                }
                last_ws_had_newline = true;
            }
            SyntaxKind::TOKEN_LET => {
                p.push_str_runtime_width_computed("let");
                p.push_signal(Signal::StartIndent);
                phase = 1;
            }
            SyntaxKind::TOKEN_IN => {
                p.push_signal(Signal::FinishIndent);
                p.push_signal(Signal::NewLine);
                p.push_str_runtime_width_computed("in");
                phase = 2;
            }
            _ => {
                if let Some(child_node) = child.as_node() {
                    match phase {
                        1 => {
                            if blank_line_pending {
                                p.push_signal(Signal::NewLine);
                                blank_line_pending = false;
                            }
                            p.push_signal(Signal::NewLine);
                            p.extend(gen_node(child_node, config));
                            has_bindings = true;
                        }
                        2 | 3 => {
                            p.push_signal(Signal::NewLine);
                            p.extend(gen_node(child_node, config));
                            phase = 3;
                        }
                        _ => {
                            p.extend(gen_node(child_node, config));
                        }
                    }
                } else if let Some(token) = child.as_token() {
                    if !super::helpers::is_trivial_token(child.kind()) {
                        if phase >= 2 {
                            p.push_signal(Signal::NewLine);
                        }
                        p.push_string(token.text().to_string());
                    }
                }
                last_ws_had_newline = false;
            }
        }
    }

    p
}
