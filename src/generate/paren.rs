use dprint_core::formatting::{ir_helpers, PrintItems, Signal};
use rnix::{SyntaxKind, SyntaxNode};

use super::gen_node;
use crate::config::NixConfig;

/// Generate IR for parenthesized expressions: `(expr)`
pub fn gen_paren(node: &SyntaxNode, config: &NixConfig) -> PrintItems {
    let mut p = PrintItems::new();

    p.push_str_runtime_width_computed("(");

    let mut inner = PrintItems::new();
    let mut has_content = false;
    let mut last_ws_had_newline = false;

    for child in node.children_with_tokens() {
        match child.kind() {
            SyntaxKind::TOKEN_L_PAREN | SyntaxKind::TOKEN_R_PAREN => {}
            SyntaxKind::TOKEN_WHITESPACE => {
                let text = child.as_token().unwrap().text();
                last_ws_had_newline = text.contains('\n');
            }
            SyntaxKind::TOKEN_COMMENT => {
                let text = child.as_token().unwrap().text().to_string();
                if !last_ws_had_newline && has_content {
                    inner.extend(super::comment::gen_trailing_comment(&text));
                } else {
                    inner.extend(super::comment::gen_comment(&text));
                }
                has_content = true;
                last_ws_had_newline = true;
            }
            _ => {
                if let Some(n) = child.as_node() {
                    inner.extend(gen_node(n, config));
                    has_content = true;
                } else if let Some(t) = child.as_token() {
                    inner.push_string(t.text().to_string());
                    has_content = true;
                }
                last_ws_had_newline = false;
            }
        }
    }

    if has_content {
        let group = ir_helpers::new_line_group({
            let mut g = PrintItems::new();
            g.push_signal(Signal::StartIndent);
            g.push_signal(Signal::SpaceOrNewLine);
            g.extend(inner);
            g.push_signal(Signal::FinishIndent);
            g.push_signal(Signal::SpaceOrNewLine);
            g
        });
        p.extend(group);
    }

    p.push_str_runtime_width_computed(")");
    p
}

/// Generate IR for string interpolation: `${expr}`
pub fn gen_interpol(node: &SyntaxNode, config: &NixConfig) -> PrintItems {
    let mut p = PrintItems::new();
    p.push_str_runtime_width_computed("${");

    for child in node.children_with_tokens() {
        match child.kind() {
            SyntaxKind::TOKEN_INTERPOL_START | SyntaxKind::TOKEN_INTERPOL_END => {}
            SyntaxKind::TOKEN_WHITESPACE => {}
            _ => {
                if let Some(n) = child.as_node() {
                    p.extend(gen_node(n, config));
                } else if let Some(t) = child.as_token() {
                    if !super::helpers::is_trivial_token(child.kind()) {
                        p.push_string(t.text().to_string());
                    }
                }
            }
        }
    }

    p.push_str_runtime_width_computed("}");
    p
}
