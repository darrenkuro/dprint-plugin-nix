use dprint_core::configuration::{ConfigKeyMap, GlobalConfiguration, NewLineKind};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NixConfig {
    pub line_width: u32,
    pub indent_width: u8,
    pub use_tabs: bool,
    pub new_line_kind: NewLineKind,
}

impl Default for NixConfig {
    fn default() -> Self {
        Self {
            line_width: 100,
            indent_width: 2,
            use_tabs: false,
            new_line_kind: NewLineKind::LineFeed,
        }
    }
}

pub struct ResolveConfigResult {
    pub config: NixConfig,
    pub diagnostics: Vec<dprint_core::configuration::ConfigurationDiagnostic>,
}

pub fn resolve_config(
    _plugin_config: ConfigKeyMap,
    global_config: &GlobalConfiguration,
) -> ResolveConfigResult {
    let diagnostics = Vec::new();

    let config = NixConfig {
        line_width: global_config.line_width.unwrap_or(100),
        indent_width: global_config.indent_width.unwrap_or(2),
        use_tabs: global_config.use_tabs.unwrap_or(false),
        new_line_kind: global_config.new_line_kind.unwrap_or(NewLineKind::LineFeed),
    };

    ResolveConfigResult { config, diagnostics }
}
