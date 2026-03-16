use crate::config::NixConfig;
use crate::generate;
use dprint_core::configuration::NewLineKind;
use dprint_core::formatting::PrintOptions;

pub fn format_text(input: &str, config: &NixConfig) -> Result<Option<String>, String> {
    let parse = rnix::Root::parse(input);

    // Check for parse errors
    let errors = parse.errors();
    if !errors.is_empty() {
        return Err(errors
            .iter()
            .map(|e| e.to_string())
            .collect::<Vec<_>>()
            .join("; "));
    }

    let root = parse.syntax();

    let new_line_text = match config.new_line_kind {
        NewLineKind::LineFeed => "\n",
        NewLineKind::CarriageReturnLineFeed => "\r\n",
        NewLineKind::Auto => {
            if input.contains("\r\n") {
                "\r\n"
            } else {
                "\n"
            }
        }
    };

    // PrintItems must be generated INSIDE the format() closure so that
    // dprint-core's thread-local ID counters are properly tracked for capacity
    let formatted = dprint_core::formatting::format(
        || generate::gen_node(&root, config),
        PrintOptions {
            indent_width: config.indent_width,
            max_width: config.line_width,
            use_tabs: config.use_tabs,
            new_line_text,
        },
    );

    if formatted == input {
        Ok(None)
    } else {
        Ok(Some(formatted))
    }
}
