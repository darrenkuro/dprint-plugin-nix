use dprint_core::formatting::{ir_helpers, PrintItems, Signal};

/// Generate IR for a standalone comment (on its own line).
/// Does NOT emit a trailing newline — the caller/container separator handles that.
pub fn gen_comment(text: &str) -> PrintItems {
    let mut p = PrintItems::new();
    let trimmed = text.trim_end();

    if trimmed.starts_with('#') {
        // Line comment — emit text, use ExpectNewLine so dprint-core knows
        // a newline should happen but won't double up with the separator's newline
        p.push_string(trimmed.to_string());
        p.push_signal(Signal::ExpectNewLine);
    } else if trimmed.starts_with("/*") {
        // Block comment — use gen_from_raw_string for multi-line preservation
        p.extend(ir_helpers::gen_from_raw_string(trimmed));
        p.push_signal(Signal::ExpectNewLine);
    }

    p
}

/// Generate IR for a trailing comment (on the same line as preceding code).
/// Pattern: `SpaceIfNotTrailing + comment_text + ExpectNewLine`
pub fn gen_trailing_comment(text: &str) -> PrintItems {
    let mut p = PrintItems::new();
    let trimmed = text.trim_end();

    p.push_signal(Signal::SpaceIfNotTrailing);
    p.push_string(trimmed.to_string());
    p.push_signal(Signal::ExpectNewLine);

    p
}
