use dprint_core::formatting::{ir_helpers, PrintItems};
use rnix::{SyntaxKind, SyntaxNode};

use super::gen_node;
use crate::config::NixConfig;

/// Generate IR for string literals: `"..."` and `''...''`
///
/// Strings are mostly preserved as-is. We format interpolation expressions
/// but leave string content untouched.
pub fn gen_string(node: &SyntaxNode, config: &NixConfig) -> PrintItems {
    let mut p = PrintItems::new();

    for child in node.children_with_tokens() {
        match child.kind() {
            SyntaxKind::TOKEN_STRING_START => {
                p.push_string(child.as_token().unwrap().text().to_string());
            }
            SyntaxKind::TOKEN_STRING_CONTENT => {
                // String content may contain newlines (indented strings).
                // gen_from_raw_string handles newline splitting + StartIgnoringIndent
                let text = child.as_token().unwrap().text();
                p.extend(ir_helpers::gen_from_raw_string(text));
            }
            SyntaxKind::TOKEN_STRING_END => {
                p.push_string(child.as_token().unwrap().text().to_string());
            }
            SyntaxKind::NODE_INTERPOL => {
                // Format the interpolation expression
                if let Some(n) = child.as_node() {
                    p.extend(super::paren::gen_interpol(n, config));
                }
            }
            _ => {
                // Preserve anything else (escape sequences, etc.)
                if let Some(t) = child.as_token() {
                    p.extend(ir_helpers::gen_from_raw_string(t.text()));
                } else if let Some(n) = child.as_node() {
                    p.extend(gen_node(n, config));
                }
            }
        }
    }

    p
}
