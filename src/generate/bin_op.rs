use dprint_core::formatting::{ir_helpers, PrintItems, Signal};
use rnix::{SyntaxKind, SyntaxNode};

use super::gen_node;
use crate::config::NixConfig;

/// Generate IR for binary operation: `a + b`, `a // b`, etc.
pub fn gen_bin_op(node: &SyntaxNode, config: &NixConfig) -> PrintItems {
    let mut p = PrintItems::new();
    let mut parts: Vec<(Option<String>, PrintItems)> = Vec::new();
    let mut current_op: Option<String> = None;
    let mut last_ws_had_newline = false;

    for child in node.children_with_tokens() {
        match child.kind() {
            SyntaxKind::TOKEN_WHITESPACE => {
                let text = child.as_token().unwrap().text();
                last_ws_had_newline = text.contains('\n');
            }
            SyntaxKind::TOKEN_COMMENT => {
                let text = child.as_token().unwrap().text().to_string();
                // Attach comment to the last operand if there is one
                if !last_ws_had_newline && !parts.is_empty() {
                    let last = &mut parts.last_mut().unwrap().1;
                    last.extend(super::comment::gen_trailing_comment(&text));
                } else if !parts.is_empty() {
                    let last = &mut parts.last_mut().unwrap().1;
                    last.push_signal(Signal::NewLine);
                    last.extend(super::comment::gen_comment(&text));
                }
                last_ws_had_newline = true;
            }
            _ => {
                if child.as_token().is_some() && is_operator(child.kind()) {
                    current_op = Some(child.as_token().unwrap().text().to_string());
                } else {
                    let inner = if let Some(n) = child.as_node() {
                        gen_node(n, config)
                    } else if let Some(t) = child.as_token() {
                        let mut items = PrintItems::new();
                        items.push_string(t.text().to_string());
                        items
                    } else {
                        continue;
                    };
                    parts.push((current_op.take(), inner));
                }
                last_ws_had_newline = false;
            }
        }
    }

    for (i, (op, operand)) in parts.into_iter().enumerate() {
        if i > 0 {
            if let Some(op_text) = op {
                p.push_str_runtime_width_computed(" ");
                p.push_string(op_text);
                p.push_signal(Signal::SpaceOrNewLine);
            } else {
                p.push_signal(Signal::SpaceOrNewLine);
            }
        }
        p.extend(operand);
    }

    ir_helpers::new_line_group(p)
}

fn is_operator(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::TOKEN_ADD
            | SyntaxKind::TOKEN_SUB
            | SyntaxKind::TOKEN_MUL
            | SyntaxKind::TOKEN_DIV
            | SyntaxKind::TOKEN_CONCAT
            | SyntaxKind::TOKEN_UPDATE
            | SyntaxKind::TOKEN_AND_AND
            | SyntaxKind::TOKEN_OR_OR
            | SyntaxKind::TOKEN_EQUAL
            | SyntaxKind::TOKEN_NOT_EQUAL
            | SyntaxKind::TOKEN_LESS
            | SyntaxKind::TOKEN_LESS_OR_EQ
            | SyntaxKind::TOKEN_MORE
            | SyntaxKind::TOKEN_MORE_OR_EQ
            | SyntaxKind::TOKEN_IMPLICATION
    )
}
