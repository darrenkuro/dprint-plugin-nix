use dprint_core::formatting::ir_helpers::{
    self, GenSeparatedValuesOptions, GeneratedValue, MultiLineOptions, SingleLineOptions,
};
use dprint_core::formatting::{PrintItems, Signal};
use rnix::SyntaxKind;

/// Create a group that tries to fit on one line, expanding to multi-line if it doesn't fit.
/// Uses gen_separated_values for proper all-or-nothing break decisions (Wadler-style groups).
pub fn gen_container(
    open: &'static str,
    close: &'static str,
    items: Vec<PrintItems>,
    separator: ContainerSeparator,
    indent_width: u8,
) -> PrintItems {
    let mut p = PrintItems::new();

    if items.is_empty() {
        p.push_str_runtime_width_computed(open);
        p.push_signal(Signal::SpaceIfNotTrailing);
        p.push_str_runtime_width_computed(close);
        return p;
    }

    p.push_str_runtime_width_computed(open);

    let single_line_separator = match separator {
        ContainerSeparator::SpaceOrNewline => {
            Signal::SpaceOrNewLine.into()
        }
        ContainerSeparator::CommaSpaceOrNewline => {
            let mut sep = PrintItems::new();
            sep.push_str_runtime_width_computed(",");
            sep.push_signal(Signal::SpaceOrNewLine);
            sep
        }
    };

    let result = ir_helpers::gen_separated_values(
        |_is_multi_line_ref| {
            items
                .into_iter()
                .map(GeneratedValue::from_items)
                .collect()
        },
        GenSeparatedValuesOptions {
            prefer_hanging: false,
            force_use_new_lines: false,
            allow_blank_lines: true,
            indent_width,
            single_line_options: SingleLineOptions {
                space_at_start: true,
                space_at_end: true,
                separator: single_line_separator,
            },
            multi_line_options: MultiLineOptions::surround_newlines_indented(),
            force_possible_newline_at_start: false,
        },
    );

    p.extend(result.items);
    p.push_str_runtime_width_computed(close);
    p
}

pub enum ContainerSeparator {
    SpaceOrNewline,
    CommaSpaceOrNewline,
}

/// Check if a SyntaxKind is a token that should be skipped (whitespace/comments)
pub fn is_trivial_token(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::TOKEN_WHITESPACE | SyntaxKind::TOKEN_COMMENT
    )
}
