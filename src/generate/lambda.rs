use dprint_core::formatting::{PrintItems, Signal};
use rnix::{SyntaxKind, SyntaxNode};

use super::gen_node;
use crate::config::NixConfig;

/// Generate IR for lambda: `param: body` or `{ pattern }: body`
pub fn gen_lambda(node: &SyntaxNode, config: &NixConfig) -> PrintItems {
    let mut p = PrintItems::new();
    let mut phase = 0; // 0=param, 1=colon, 2=body
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
                        if child.kind() == SyntaxKind::TOKEN_COLON {
                            p.push_str_runtime_width_computed(":");
                            // Don't emit SpaceOrNewLine yet — defer until body
                            // so trailing comments on `:` don't get double-spaced
                            phase = 1;
                        } else if let Some(n) = child.as_node() {
                            p.extend(gen_node(n, config));
                        } else if let Some(t) = child.as_token() {
                            p.push_string(t.text().to_string());
                        }
                    }
                    1 | 2 => {
                        if phase == 1 {
                            p.push_signal(Signal::SpaceOrNewLine);
                        }
                        if let Some(n) = child.as_node() {
                            p.extend(gen_node(n, config));
                        } else if let Some(t) = child.as_token() {
                            p.push_string(t.text().to_string());
                        }
                        phase = 2;
                    }
                    _ => {}
                }
                last_ws_had_newline = false;
            }
        }
    }

    p
}

/// Generate IR for pattern: `{ a, b ? default, ... }`
pub fn gen_pattern(node: &SyntaxNode, config: &NixConfig) -> PrintItems {
    let mut items_list: Vec<PrintItems> = Vec::new();
    let mut seen_open = false;
    let mut at_binding_before: Option<PrintItems> = None;
    let mut at_binding_after: Option<PrintItems> = None;
    let mut after_close = false;
    let mut blank_line_pending = false;
    let mut last_ws_had_newline = false;

    for child in node.children_with_tokens() {
        match child.kind() {
            SyntaxKind::TOKEN_L_BRACE => {
                seen_open = true;
            }
            SyntaxKind::TOKEN_R_BRACE => {
                after_close = true;
            }
            SyntaxKind::TOKEN_WHITESPACE => {
                if seen_open && !after_close {
                    let text = child.as_token().unwrap().text();
                    let newlines = text.chars().filter(|c| *c == '\n').count();
                    last_ws_had_newline = newlines > 0;
                    if newlines > 1 {
                        blank_line_pending = true;
                    }
                }
            }
            SyntaxKind::TOKEN_COMMENT => {
                if seen_open && !after_close {
                    let text = child.as_token().unwrap().text().to_string();
                    if !last_ws_had_newline && !items_list.is_empty() {
                        let last = items_list.last_mut().unwrap();
                        last.extend(super::comment::gen_trailing_comment(&text));
                    } else {
                        let mut comment_items = PrintItems::new();
                        if blank_line_pending {
                            comment_items.push_signal(Signal::NewLine);
                            blank_line_pending = false;
                        }
                        comment_items.extend(super::comment::gen_comment(&text));
                        items_list.push(comment_items);
                    }
                    last_ws_had_newline = true;
                }
            }
            SyntaxKind::TOKEN_COMMA => {} // Handled by container separator
            SyntaxKind::TOKEN_AT => {}
            SyntaxKind::TOKEN_ELLIPSIS => {
                let mut item = PrintItems::new();
                if blank_line_pending && !items_list.is_empty() {
                    item.push_signal(Signal::NewLine);
                    blank_line_pending = false;
                }
                item.push_str_runtime_width_computed("...");
                items_list.push(item);
                last_ws_had_newline = false;
            }
            _ => {
                if !seen_open && !after_close {
                    // Before `{` — this is `name @` binding
                    if let Some(t) = child.as_token() {
                        let mut binding = PrintItems::new();
                        binding.push_string(t.text().to_string());
                        binding.push_str_runtime_width_computed(" @ ");
                        at_binding_before = Some(binding);
                    }
                } else if after_close {
                    // After `}` — this is `@ name` binding
                    if let Some(t) = child.as_token() {
                        let mut binding = PrintItems::new();
                        binding.push_str_runtime_width_computed(" @ ");
                        binding.push_string(t.text().to_string());
                        at_binding_after = Some(binding);
                    }
                } else if let Some(child_node) = child.as_node() {
                    let mut item = PrintItems::new();
                    if blank_line_pending && !items_list.is_empty() {
                        item.push_signal(Signal::NewLine);
                        blank_line_pending = false;
                    }
                    item.extend(gen_node(child_node, config));
                    items_list.push(item);
                } else if let Some(t) = child.as_token() {
                    if !super::helpers::is_trivial_token(child.kind()) {
                        let mut item = PrintItems::new();
                        item.push_string(t.text().to_string());
                        items_list.push(item);
                    }
                }
                last_ws_had_newline = false;
            }
        }
    }

    let mut p = PrintItems::new();

    if let Some(binding) = at_binding_before {
        p.extend(binding);
    }

    p.extend(super::helpers::gen_container(
        "{",
        "}",
        items_list,
        super::helpers::ContainerSeparator::CommaSpaceOrNewline,
        config.indent_width,
    ));

    if let Some(binding) = at_binding_after {
        p.extend(binding);
    }

    p
}

/// Generate IR for pattern entry: `name` or `name ? default`
pub fn gen_pat_entry(node: &SyntaxNode, config: &NixConfig) -> PrintItems {
    let mut p = PrintItems::new();

    for child in node.children_with_tokens() {
        match child.kind() {
            SyntaxKind::TOKEN_WHITESPACE => {}
            SyntaxKind::TOKEN_COMMENT => {
                let text = child.as_token().unwrap().text().to_string();
                p.extend(super::comment::gen_trailing_comment(&text));
            }
            SyntaxKind::TOKEN_QUESTION => {
                p.push_str_runtime_width_computed(" ? ");
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
