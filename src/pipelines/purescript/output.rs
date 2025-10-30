use super::super::prank_id_selector;
use crate::{
    common::{html_rewrite::Document, nonce_attr},
    config::{rt::RtcBuild, types::CrossOrigin},
    pipelines::purescript::{sri::SriBuilder, PureScriptAppType},
};
use anyhow::bail;
use std::{collections::HashMap, sync::Arc};

/// The output of a PureScript build pipeline.
#[derive(Debug)]
pub struct PureScriptAppOutput {
    /// The runtime build config.
    pub cfg: Arc<RtcBuild>,
    /// The ID of this pipeline.
    pub id: Option<usize>,
    /// The filename of the generated JS bundle file written to the dist dir.
    pub bundle_output: String,
    /// Is this module main or a worker.
    pub r#type: PureScriptAppType,
    /// The cross-origin setting for loading the resources
    pub cross_origin: CrossOrigin,
    /// The output digests for the sub-resources
    pub integrities: SriBuilder,
    /// The target of the initializer module
    pub initializer: Option<String>,
    /// An optional script to run the main module in dev mode.
    pub dev_mode_run_script: Option<String>,
    // REMOVED: js_output, wasm_output, wasm_size, import_bindings, import_bindings_name
}

pub fn pattern_evaluate(template: &str, params: &HashMap<String, String>) -> String {
    let mut result = template.to_string();
    for (k, v) in params.iter() {
        let pattern = format!("{{{}}}", k.as_str());
        if let Some(file_path) = v.strip_prefix('@') {
            if let Ok(contents) = std::fs::read_to_string(file_path) {
                result = str::replace(result.as_str(), &pattern, contents.as_str());
            }
        } else {
            result = str::replace(result.as_str(), &pattern, v);
        }
    }
    result
}

impl PureScriptAppOutput {
    pub async fn finalize(self, dom: &mut Document) -> anyhow::Result<()> {
        if !self.cfg.inject_scripts {
            // Configuration directed we do not inject any scripts.
            return Ok(());
        }

        let (base, bundle, head, body) = (
            &self.cfg.public_url,
            &self.bundle_output, // CHANGED
            "html head",
            "html body",
        );
        // The code to fire the `PrankApplicationStarted` event.
        let fire = r#"dispatchEvent(new CustomEvent("PrankApplicationStarted"));"#;
        let (pattern_script, pattern_preload) =
            (&self.cfg.pattern_script, &self.cfg.pattern_preload);
        let mut params = self.cfg.pattern_params.clone();
        params.insert("base".to_owned(), base.to_string());
        params.insert("bundle".to_owned(), bundle.clone()); // CHANGED
                                                            // REMOVED: wasm param
        params.insert("crossorigin".to_owned(), self.cross_origin.to_string());

        if let Some(pattern) = pattern_preload {
            dom.append_html(head, &pattern_evaluate(pattern, &params))?;
        } else {
            self.integrities.clone().build().inject(
                dom,
                head,
                base,
                self.cross_origin,
                &self.cfg.create_nonce,
            )?;
        }

        let script = if let Some(dev_script) = self.dev_mode_run_script {
            format!(
                r#"
<script type="module"{nonce}>
{dev_script}
{fire}
</script>"#,
                nonce = nonce_attr(&self.cfg.create_nonce),
                dev_script = dev_script,
                fire = fire
            )
        } else {
            match pattern_script {
                Some(pattern) => pattern_evaluate(pattern, &params),
                None => self.default_initializer(base, bundle, fire), // CHANGED
            }
        };

        match self.id {
            Some(id) => dom.replace_with_html(&prank_id_selector(id), &script)?,
            None => {
                if dom.len(body)? == 0 {
                    bail!(
                        r#"Document has neither a <link data-prank rel="purescript"/> nor a <body>. Either one must be present."#
                    );
                }
                dom.append_html(body, &script)?
            }
        }

        Ok(())
    }

    /// create the default initializer script section
    fn default_initializer(&self, base: &str, bundle: &str, fire: &str) -> String {
        // REWRITTEN: This function is now much simpler and handles JS modules.
        let nonce = nonce_attr(&self.cfg.create_nonce);

        match &self.initializer {
            None => format!(
                r#"
<script type="module"{nonce}>
import '{base}{bundle}';
{fire}
</script>"#,
                nonce = nonce,
                base = base,
                bundle = bundle,
                fire = fire
            ),
            Some(initializer) => format!(
                r#"
<script type="module"{nonce}>
import setup from '{base}{initializer}';
if (typeof setup === 'function') {{
    await Promise.resolve(setup());
}}
import '{base}{bundle}';
{fire}
</script>"#,
                nonce = nonce,
                base = base,
                initializer = initializer,
                bundle = bundle,
                fire = fire
            ),
        }
    }
}
