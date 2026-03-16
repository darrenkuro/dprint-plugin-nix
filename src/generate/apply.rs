use dprint_core::formatting::{ir_helpers, PrintItems, Signal};
use rnix::{SyntaxKind, SyntaxNode};

use super::gen_node;
use crate::config::NixConfig;

/// Generate IR for function application: `f x` or `f x y z`
pub fn gen_apply(node: &SyntaxNode, config: &NixConfig) -> PrintItems {
    let mut p = PrintItems::new();
    let mut first = true;
    let mut last_ws_had_newline = false;

    for child in node.children_with_tokens() {
        match child.kind() {
            SyntaxKind::TOKEN_WHITESPACE => {
                let text = child.as_token().unwrap().text();
                last_ws_had_newline = text.contains('\n');
            }
            SyntaxKind::TOKEN_COMMENT => {
                let text = child.as_token().unwrap().text().to_string();
                if !last_ws_had_newline && !first {
                    p.extend(super::comment::gen_trailing_comment(&text));
                } else {
                    p.extend(super::comment::gen_comment(&text));
                }
                last_ws_had_newline = true;
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

                if first {
                    p.extend(inner);
                    first = false;
                } else {
                    p.push_signal(Signal::SpaceOrNewLine);
                    p.extend(inner);
                }
                last_ws_had_newline = false;
            }
        }
    }

    ir_helpers::new_line_group(p)
}
