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

/// One element inside a pattern's braces: an entry (`a`, `b ? 1`, `...`) or a
/// standalone comment. Trailing comments are kept apart so the comma can go before them.
struct PatItem {
    body: PrintItems,
    trailing: PrintItems,
    is_entry: bool,
}

/// Generate IR for pattern: `{ a, b ? default, ... }`
pub fn gen_pattern(node: &SyntaxNode, config: &NixConfig) -> PrintItems {
    let mut pat_items: Vec<PatItem> = Vec::new();
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
                    if !last_ws_had_newline && !pat_items.is_empty() {
                        let last = pat_items.last_mut().unwrap();
                        last.trailing
                            .extend(super::comment::gen_trailing_comment(&text));
                    } else {
                        let mut body = PrintItems::new();
                        if blank_line_pending {
                            body.push_signal(Signal::NewLine);
                            blank_line_pending = false;
                        }
                        body.extend(super::comment::gen_comment(&text));
                        pat_items.push(PatItem {
                            body,
                            trailing: PrintItems::new(),
                            is_entry: false,
                        });
                    }
                    last_ws_had_newline = true;
                }
            }
            SyntaxKind::TOKEN_COMMA => {} // Re-emitted after each entry below
            SyntaxKind::NODE_PAT_BIND => {
                // `name @` before `{`, or `@ name` after `}`
                let name = child
                    .as_node()
                    .unwrap()
                    .children()
                    .find(|n| n.kind() == SyntaxKind::NODE_IDENT)
                    .map(|n| n.text().to_string())
                    .unwrap_or_default();
                let mut binding = PrintItems::new();
                if after_close {
                    binding.push_str_runtime_width_computed(" @ ");
                    binding.push_string(name);
                    at_binding_after = Some(binding);
                } else {
                    binding.push_string(name);
                    binding.push_str_runtime_width_computed(" @ ");
                    at_binding_before = Some(binding);
                }
            }
            _ => {
                if seen_open && !after_close {
                    let mut body = PrintItems::new();
                    if blank_line_pending && !pat_items.is_empty() {
                        body.push_signal(Signal::NewLine);
                    }
                    blank_line_pending = false;
                    if let Some(child_node) = child.as_node() {
                        body.extend(gen_node(child_node, config));
                    } else if let Some(t) = child.as_token() {
                        // TOKEN_ELLIPSIS
                        body.push_string(t.text().to_string());
                    }
                    pat_items.push(PatItem {
                        body,
                        trailing: PrintItems::new(),
                        is_entry: true,
                    });
                }
                last_ws_had_newline = false;
            }
        }
    }

    // Commas live inside each entry (not in the container separator): dprint-core only
    // emits the separator in single-line mode, so a wrapped pattern would lose them.
    let last_entry = pat_items.iter().rposition(|item| item.is_entry);
    let items_list = pat_items
        .into_iter()
        .enumerate()
        .map(|(i, item)| {
            let mut p = item.body;
            if item.is_entry && Some(i) != last_entry {
                p.push_str_runtime_width_computed(",");
            }
            p.extend(item.trailing);
            p
        })
        .collect();

    let mut p = PrintItems::new();

    if let Some(binding) = at_binding_before {
        p.extend(binding);
    }

    p.extend(super::helpers::gen_container(
        "{",
        "}",
        items_list,
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
