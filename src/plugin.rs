use dprint_core::configuration::{ConfigKeyMap, GlobalConfiguration};
use dprint_core::plugins::{
    CheckConfigUpdatesMessage, ConfigChange, FileMatchingInfo, FormatResult, PluginInfo,
    PluginResolveConfigurationResult, SyncFormatRequest, SyncHostFormatRequest,
    SyncPluginHandler,
};

use crate::config::{self, NixConfig};
use crate::format::format_text;

pub struct NixPluginHandler;

impl SyncPluginHandler<NixConfig> for NixPluginHandler {
    fn plugin_info(&mut self) -> PluginInfo {
        PluginInfo {
            name: String::from("nix"),
            version: env!("CARGO_PKG_VERSION").to_string(),
            config_key: String::from("nix"),
            help_url: String::from("https://github.com/darrenkuro/dprint-plugin-nix"),
            config_schema_url: String::new(),
            update_url: None,
        }
    }

    fn license_text(&mut self) -> String {
        String::from("MIT License")
    }

    fn resolve_config(
        &mut self,
        config: ConfigKeyMap,
        global_config: &GlobalConfiguration,
    ) -> PluginResolveConfigurationResult<NixConfig> {
        let result = config::resolve_config(config, global_config);

        PluginResolveConfigurationResult {
            config: result.config,
            file_matching: FileMatchingInfo {
                file_extensions: vec![String::from("nix")],
                file_names: vec![],
            },
            diagnostics: result.diagnostics,
        }
    }

    fn check_config_updates(
        &self,
        _message: CheckConfigUpdatesMessage,
    ) -> anyhow::Result<Vec<ConfigChange>> {
        Ok(vec![])
    }

    fn format(
        &mut self,
        request: SyncFormatRequest<NixConfig>,
        _format_with_host: impl FnMut(SyncHostFormatRequest) -> FormatResult,
    ) -> FormatResult {
        let input = String::from_utf8(request.file_bytes)
            .map_err(|e| anyhow::anyhow!("Invalid UTF-8: {}", e))?;

        match format_text(&input, request.config) {
            Ok(Some(formatted)) => Ok(Some(formatted.into_bytes())),
            Ok(None) => Ok(None),
            Err(e) => Err(anyhow::anyhow!("{}", e)),
        }
    }
}

use dprint_core::generate_plugin_code;
generate_plugin_code!(NixPluginHandler, NixPluginHandler, NixConfig);
