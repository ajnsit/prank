use crate::config::models::ConfigModel;
use crate::config::Configuration;
use clap::Args;
use schemars::JsonSchema;
use serde::Deserialize;

/// Config options for automatic application downloads.
// **NOTE:** As there are no differences between the persistent configuration and the CLI overrides
// at all, this struct is used for both configuration as well as CLI arguments.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Args, JsonSchema)]
#[command(next_help_heading = "Tools")]
pub struct Tools {
    /// Version of `dart-sass` to use.
    #[serde(default)]
    #[arg(env = "PRANK_TOOLS_SASS")]
    pub sass: Option<String>,

    /// Version of `tailwindcss-cli` to use.
    #[serde(default)]
    #[arg(env = "PRANK_TOOLS_TAILWINDCSS")]
    pub tailwindcss: Option<String>,

    /// Version of `purescript-backend-es` to use.
    #[serde(default)]
    #[arg(env = "PRANK_TOOLS_PURESCRIPT_BACKEND_ES")]
    pub purescript_backend_es: Option<String>,

    /// Version of `spago` to use.
    #[serde(default)]
    #[arg(env = "PRANK_TOOLS_SPAGO")]
    pub spago: Option<String>,

    /// Version of `purs-tidy` to use.
    #[serde(default)]
    #[arg(env = "PRANK_TOOLS_PURS_TIDY")]
    pub purs_tidy: Option<String>,
}

impl Tools {
    pub fn apply_to(self, mut config: Configuration) -> anyhow::Result<Configuration> {
        config.tools.sass = self.sass.or(config.tools.sass);
        config.tools.tailwindcss = self.tailwindcss.or(config.tools.tailwindcss);
        config.tools.purescript_backend_es = self
            .purescript_backend_es
            .or(config.tools.purescript_backend_es);
        config.tools.spago = self.spago.or(config.tools.spago);
        config.tools.purs_tidy = self.purs_tidy.or(config.tools.purs_tidy);
        Ok(config)
    }
}

impl ConfigModel for Tools {}
