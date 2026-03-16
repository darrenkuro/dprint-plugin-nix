use dprint_core::formatting::{PrintItems, Signal};
use rnix::{SyntaxKind, SyntaxNode};

use super::helpers::{gen_container, ContainerSeparator};
use super::gen_node;
use crate::config::NixConfig;

pub fn gen_list(node: &SyntaxNode, config: &NixConfig) -> PrintItems {
    let mut items_list: Vec<PrintItems> = Vec::new();
    let mut seen_open = false;
    let mut blank_line_pending = false;
    // Track whether the last whitespace token contained a newline.
    // Used to distinguish trailing comments (same line) from standalone comments.
    let mut last_ws_had_newline = false;

    for child in node.children_with_tokens() {
        match child.kind() {
            SyntaxKind::TOKEN_L_BRACK => {
                seen_open = true;
            }
            SyntaxKind::TOKEN_R_BRACK => {}
            SyntaxKind::TOKEN_WHITESPACE => {
                if seen_open {
                    let text = child.as_token().unwrap().text();
                    let newlines = text.chars().filter(|c| *c == '\n').count();
                    last_ws_had_newline = newlines > 0;
                    if newlines > 1 {
                        blank_line_pending = true;
                    }
                }
            }
            SyntaxKind::TOKEN_COMMENT => {
                let text = child.as_token().unwrap().text().to_string();

                if !last_ws_had_newline && !items_list.is_empty() {
                    // Trailing comment — attach to the previous item
                    let last = items_list.last_mut().unwrap();
                    last.extend(super::comment::gen_trailing_comment(&text));
                } else {
                    // Standalone/leading comment — its own item
                    let mut comment_items = PrintItems::new();
                    if blank_line_pending {
                        comment_items.push_signal(Signal::NewLine);
                        blank_line_pending = false;
                    }
                    comment_items.extend(super::comment::gen_comment(&text));
                    items_list.push(comment_items);
                }
                // After a line comment, the next token is on a new line
                last_ws_had_newline = true;
            }
            _ => {
                if let Some(child_node) = child.as_node() {
                    let mut item = PrintItems::new();
                    if blank_line_pending && !items_list.is_empty() {
                        item.push_signal(Signal::NewLine);
                        blank_line_pending = false;
                    }
                    item.extend(gen_node(child_node, config));
                    items_list.push(item);
                } else if let Some(token) = child.as_token() {
                    if !super::helpers::is_trivial_token(child.kind()) {
                        let mut item = PrintItems::new();
                        if blank_line_pending && !items_list.is_empty() {
                            item.push_signal(Signal::NewLine);
                            blank_line_pending = false;
                        }
                        item.push_string(token.text().to_string());
                        items_list.push(item);
                    }
                }
                last_ws_had_newline = false;
            }
        }
    }

    gen_container("[", "]", items_list, ContainerSeparator::SpaceOrNewline, config.indent_width)
}
