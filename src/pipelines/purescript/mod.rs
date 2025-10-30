mod output;
mod sri;

pub use output::PureScriptAppOutput;

use super::{data_target_path, Attrs, PrankAssetPipelineOutput, ATTR_HREF};
use crate::{
    common::{self, path_exists},
    config::{
        rt::{Features, RtcBuild},
        types::CrossOrigin,
    },
    pipelines::purescript::sri::{SriBuilder, SriOptions, SriType},
    processing::{integrity::IntegrityType, minify::minify_js},
    tools::{self, Application},
};
use anyhow::{anyhow, bail, ensure, Context, Result};
use minify_js::TopLevelMode;
use seahash::SeaHasher;
use serde::Deserialize;
use std::{
    fs::File,
    hash::Hasher,
    io::BufReader,
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
};
use tokio::{fs, sync::mpsc, task::JoinHandle};
use tracing::log;

#[derive(Debug, Clone, Deserialize)]
pub struct SpagoMetadata {
    #[serde(skip)]
    /// The path to the spago.yaml manifest
    pub manifest_path: PathBuf,
    #[serde(skip)]
    /// The directory containing the manifest
    pub workspace_root: PathBuf,
    /// The `package` section of the spago.yaml
    pub package: SpagoPackage,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SpagoPackage {
    pub name: String,
}

impl SpagoMetadata {
    /// Create a new instance by parsing a spago.yaml file.
    pub async fn new(manifest_path: &Path) -> Result<Self> {
        let file = File::open(manifest_path)
            .with_context(|| format!("error opening spago manifest {}", manifest_path.display()))?;
        let reader = BufReader::new(file);
        let mut metadata: SpagoMetadata = serde_yaml::from_reader(reader)
            .with_context(|| format!("error parsing spago manifest {}", manifest_path.display()))?;

        metadata.manifest_path = manifest_path.to_path_buf();
        metadata.workspace_root = manifest_path
            .parent()
            .ok_or_else(|| anyhow!("manifest path has no parent directory"))?
            .to_path_buf();

        Ok(metadata)
    }

    /// Spago's default output directory, relative to the workspace root.
    pub fn target_directory(&self) -> PathBuf {
        self.workspace_root.join("output")
    }
}

/// A PureScript application pipeline.
pub struct PureScriptApp {
    /// The ID of this pipeline's source HTML element.
    id: Option<usize>,
    /// Runtime config.
    cfg: Arc<RtcBuild>,
    /// Skip building
    skip_build: bool,
    /// Spago profile (for `spago build`)
    spago_profile: Option<String>,
    /// The configuration of the features passed to spago.
    spago_features: Features,
    /// Is this module main or a worker?
    app_type: PureScriptAppType,
    /// All metadata associated with the target Spago project.
    manifest: SpagoMetadata,
    /// An optional channel to be used to communicate paths to ignore back to the watcher.
    ignore_chan: Option<mpsc::Sender<Vec<PathBuf>>>,
    /// The main module to bundle (e.g., "Main").
    main_module: Option<String>,
    /// Optional target path inside the dist dir.
    target_path: Option<PathBuf>,
    /// Cross-origin setting for resources
    cross_origin: CrossOrigin,
    /// Subresource integrity builder
    sri: SriBuilder,
    /// The name of the initializer module
    initializer: Option<PathBuf>,
    /// Paths that have changed in the current build cycle.
    changed_paths: Vec<PathBuf>,
}

/// Describes how the purescript application is used.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PureScriptAppType {
    /// Used as the main application.
    Main,
    // /// Used as a web worker.
    // Worker,
}

impl FromStr for PureScriptAppType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "main" => Ok(PureScriptAppType::Main),
            _ => bail!(
                r#"unknown `data-type="{}"` value for <link data-prank rel="purescript" .../> attr; please ensure the value is lowercase and is a supported type"#,
                s
            ),
        }
    }
}

impl PureScriptApp {
    pub const TYPE_PURESCRIPT_APP: &'static str = "purescript";

    pub async fn new(
        cfg: Arc<RtcBuild>,
        html_dir: Arc<PathBuf>,
        ignore_chan: Option<mpsc::Sender<Vec<PathBuf>>>,
        attrs: Attrs,
        id: usize,
        changed_paths: Vec<PathBuf>,
    ) -> Result<Self> {
        let manifest_href = attrs
            .get(ATTR_HREF)
            .map(|attr| {
                let mut path = PathBuf::new();
                path.extend(attr.split('/'));
                if !path.is_absolute() {
                    path = html_dir.join(path);
                }
                if !path.ends_with("spago.yaml") {
                    path = path.join("spago.yaml");
                }
                path
            })
            .unwrap_or_else(|| html_dir.join("spago.yaml"));

        let main_module = attrs.get("data-main").map(|attr| attr.to_string());

        let app_type = attrs
            .get("data-type")
            .map(|attr| attr.parse())
            .transpose()?
            .unwrap_or(PureScriptAppType::Main);
        let cross_origin = attrs
            .get("data-cross-origin")
            .map(|attr| CrossOrigin::from_str(attr))
            .transpose()?
            .unwrap_or_default();
        let integrity = IntegrityType::from_attrs(&attrs, &cfg)?;

        let manifest = SpagoMetadata::new(&manifest_href).await?;
        let id = Some(id);

        // spago profile
        let data_spago_profile = match cfg.release {
            true => attrs.get("data-spago-profile-release"),
            false => attrs.get("data-spago-profile-dev"),
        }
        .or_else(|| attrs.get("data-spago-profile"));

        let spago_profile = match data_spago_profile {
            Some(spago_profile) => {
                let spago_profile = &spago_profile.value;
                if let Some(config_spago_profile) = &cfg.spago_profile {
                    log::warn!("Spago profile from configuration ({config_spago_profile}) will be overridden with HTML file's more specific setting ({spago_profile})");
                }
                Some(spago_profile.clone())
            }
            None => cfg.spago_profile.as_ref().cloned(),
        };

        // spago features
        let data_features = attrs
            .get("data-spago-features")
            .map(|attr| attr.to_string());
        let data_all_features = attrs.contains_key("data-spago-all-features");
        let data_no_default_features = attrs.contains_key("data-spago-no-default-features");

        ensure!(
            !(data_all_features && (data_no_default_features || data_features.is_some())),
            "Cannot combine --all-features with --no-default-features and/or --features"
        );

        let spago_features = if data_all_features {
            Features::All
        } else if data_no_default_features || data_features.is_some() {
            Features::Custom {
                features: data_features,
                no_default_features: data_no_default_features,
            }
        } else {
            cfg.spago_features.clone()
        };

        // skip
        let skip_build = attrs.contains_key("data-prank-skip");

        // progress function
        let initializer = attrs
            .get("data-initializer")
            .map(|path| PathBuf::from_str(path))
            .transpose()?
            .map(|path| {
                if !path.is_absolute() {
                    html_dir.join(path)
                } else {
                    path
                }
            });

        let target_path = data_target_path(&attrs)?;

        // done
        Ok(Self {
            id,
            cfg,
            skip_build,
            spago_profile,
            spago_features,
            manifest,
            ignore_chan,
            main_module,
            target_path,
            app_type,
            cross_origin,
            sri: SriBuilder::new(integrity),
            initializer,
            changed_paths,
        })
    }

    /// Create a new instance from reasonable defaults
    ///
    /// This will return `Ok(None)` in case no `spago.yaml` was found. And fail in any case
    /// where no default could be evaluated.
    pub async fn new_default(
        cfg: Arc<RtcBuild>,
        html_dir: Arc<PathBuf>,
        ignore_chan: Option<mpsc::Sender<Vec<PathBuf>>>,
    ) -> Result<Option<Self>> {
        let path = html_dir.join("spago.yaml");

        if !tokio::fs::try_exists(&path).await? {
            // no spago.yaml found, don't assume a project
            return Ok(None);
        }

        let manifest = SpagoMetadata::new(&path).await?;
        let integrity = IntegrityType::default_unless(cfg.no_sri);

        Ok(Some(Self {
            id: None,
            skip_build: false,
            spago_features: cfg.spago_features.clone(),
            spago_profile: cfg.spago_profile.clone(),
            cfg,
            manifest,
            ignore_chan,
            main_module: None,
            target_path: None,
            app_type: PureScriptAppType::Main,
            cross_origin: Default::default(),
            sri: SriBuilder::new(integrity),
            initializer: None,
            changed_paths: vec![],
        }))
    }

    /// Spawn a new pipeline.
    #[tracing::instrument(level = "trace", skip(self))]
    pub fn spawn(self) -> JoinHandle<Result<PrankAssetPipelineOutput>> {
        tokio::spawn(self.build())
    }

    #[tracing::instrument(level = "trace", skip(self))]
    async fn build(mut self) -> Result<PrankAssetPipelineOutput> {
        if self.skip_build {
            return Ok(PrankAssetPipelineOutput::None);
        }

        // 1. Conditionally run `spago build`
        if self.should_run_spago_build() {
            self.spago_build().await.context("running spago build")?;
        } else {
            tracing::debug!("Skipping spago build as no relevant PureScript or FFI files changed.");
        }

        let (bundle_name, bundle_dest_path): (String, PathBuf);
        let mut dev_mode_run_script_option: Option<String> = None;

        if self.cfg.release {
            // RELEASE MODE
            // 2. Run `purs-backend-es bundle`

            let bundle_path = self
                .purs_bundle()
                .await
                .context("running purs-backend-es bundle-app")?;

            // 3. Hash, copy, and minify the single bundle
            let hashed_bundle_name = self.hashed_name(&bundle_path).await?;
            let dest_path = self.cfg.staging_dist.join(&hashed_bundle_name);

            self.copy_or_minify_js(&bundle_path, &dest_path, TopLevelMode::Module)
                .await
                .context("error minifying or copying JS bundle")?;

            bundle_name = hashed_bundle_name;
            bundle_dest_path = dest_path;
        } else {
            // DEV MODE
            // 2. We assume the server is configured to serve `manifest.target_directory()`
            // at the URL path `/output`.

            // 3. Determine the entry point path.
            let main_module = self.main_module.as_deref().unwrap_or("Main");
            // The relative path to the entry file (e.g., "output/Main/index.js")
            let entry_path = PathBuf::from("output").join(main_module).join("index.js");

            // The full path to the *original file* in the project's output dir
            bundle_dest_path = self
                .manifest
                .target_directory()
                .join(main_module)
                .join("index.js");

            if !path_exists(&bundle_dest_path).await? {
                bail!("spago build succeeded, but main module entry point was not found at {}. Is `data-main` attribute correct?", bundle_dest_path.display());
            }

            // The "name" to pass to the HTML is the relative path, using URL separators.
            bundle_name = entry_path.to_string_lossy().replace('\\', "/");

            // The script to run the main module in dev mode.
            let dev_mode_run_script = format!(
                r#"import {{ main }} from '/{bundle_name}';
main();"#
            );

            dev_mode_run_script_option = Some(dev_mode_run_script);
        }

        // 4. Build SRI for the entry point (bundle or main.js) and create output
        let output = self
            .build_sri_and_output(&bundle_name, &bundle_dest_path, dev_mode_run_script_option)
            .await
            .context("processing final JS")?;

        tracing::debug!("purescript build complete");
        Ok(PrankAssetPipelineOutput::PureScriptApp(output))
    }

    /// Run `spago build` to compile .purs to .js
    #[tracing::instrument(level = "trace", skip(self))]
    async fn spago_build(&mut self) -> Result<()> {
        tracing::debug!("building {}", &self.manifest.package.name);

        // Spawn the spago build process.
        let mut args = vec!["build"];

        if let Some(_profile) = &self.spago_profile {
            // No profiles
            // args.push("--profile");
            // args.push(profile);
        } else if self.cfg.release {
            // We build release bundles via purs-backend-es
            // args.push("--release");
        }
        if self.cfg.offline {
            args.push("--offline");
        }
        if self.cfg.frozen {
            // Frozen is the default
            // args.push("--frozen");
        }
        if self.cfg.locked {
            args.push("--pure");
        }

        let build_res = common::run_command("spago", "spago", &args, &self.cfg.working_directory)
            .await
            .context("error during spago build execution");

        // Send spago's target dir over to the watcher to be ignored.
        if let Some(chan) = &mut self.ignore_chan {
            if let Ok(target_dir) = self.manifest.target_directory().canonicalize() {
                let target_dir_recursive = target_dir.join("**");
                let _ = chan.try_send(vec![target_dir, target_dir_recursive]);
            }
        }

        build_res?;
        Ok(())
    }

    /// Check if any of the changed paths are PureScript or FFI files.
    fn should_run_spago_build(&self) -> bool {
        if self.changed_paths.is_empty() {
            return true;
        }
        for path in &self.changed_paths {
            if path.extension().is_some_and(|ext| ext == "purs") {
                return true;
            }
            // Check for JS files in `src/` (assuming FFI files are here)
            if path.extension().is_some_and(|ext| ext == "js")
                && path.components().any(|c| c.as_os_str() == "src")
            {
                return true;
            }
        }
        false
    }

    /// Run `purs-backend-es` to create a JS bundle (for release mode).
    #[tracing::instrument(level = "trace", skip(self))]
    async fn purs_bundle(&self) -> Result<PathBuf> {
        let bundle_dir = self.manifest.target_directory().join("bundle");
        fs::create_dir_all(&bundle_dir)
            .await
            .context("error creating bundle output directory")?;

        let bundle_path = bundle_dir.join("index.js");
        let bundle_path_str = bundle_path.to_string_lossy();

        let args = vec!["bundle-app", "--to", &bundle_path_str];

        tracing::debug!("bundling with purs-backend-es");

        let version = self.cfg.tools.purescript_backend_es.as_deref();
        let purs_backend_es = tools::get(
            Application::PureScriptBackendEs,
            version,
            self.cfg.offline,
            &self.cfg.client_options(),
        )
        .await?;

        common::run_command(
            Application::PureScriptBackendEs.name(),
            &purs_backend_es,
            &args,
            &self.cfg.core.working_directory,
        )
        .await?;

        if !path_exists(&bundle_path).await? {
            bail!(
                "purs-backend-es succeeded, but expected JS bundle was not found at {}. ",
                bundle_path.display()
            );
        }

        Ok(bundle_path)
    }

    /// create a cache busting hashed name based on a path, if enabled
    async fn hashed_name(&self, path: impl AsRef<Path>) -> Result<String> {
        let path = path.as_ref();
        let name = path
            .file_name()
            .ok_or_else(|| anyhow!("Must be a file: {}", path.display()))?
            .to_string_lossy()
            .to_string();

        Ok(self
            .hashed(path)
            .await?
            .map(|hashed| format!("{hashed}-{name}"))
            .unwrap_or_else(|| name.clone()))
    }

    /// create a cache busting string, if enabled
    async fn hashed(&self, path: &Path) -> Result<Option<String>> {
        // generate a hashed name, just for cache busting
        Ok(match self.cfg.filehash {
            false => None,
            true => {
                tracing::debug!("processing hash for {}", path.display());

                let hash = {
                    let path = path.to_owned();
                    tokio::task::spawn_blocking(move || {
                        let mut file = std::fs::File::open(&path)?;
                        let mut hasher = SeaHasher::new();
                        std::io::copy(&mut file, &mut hasher).with_context(|| {
                            format!("error reading '{}' for hash generation", path.display())
                        })?;
                        Ok::<_, anyhow::Error>(hasher.finish())
                    })
                    .await??
                };

                Some(format!("{hash:x}"))
            }
        })
    }

    async fn copy_or_minify_js(
        &self,
        origin_path: impl AsRef<Path>,
        destination_path: &Path,
        mode: TopLevelMode,
    ) -> Result<()> {
        let bytes = fs::read(origin_path)
            .await
            .context("error reading JS loader file")?;

        let write_bytes = match self.cfg.should_minify() {
            true => minify_js(bytes, mode),
            false => bytes,
        };

        fs::write(destination_path, write_bytes)
            .await
            .context("error writing JS loader file to stage dir")?;

        Ok(())
    }

    /// Build the final SRI digests and construct the output object
    #[tracing::instrument(level = "trace", skip(self))]
    async fn build_sri_and_output(
        &self,
        bundle_name: &str,  // In debug, this is a path like "output/Main/index.js"
        bundle_path: &Path, // This is the full path to the file in the staging dir
        dev_mode_run_script: Option<String>,
    ) -> Result<PureScriptAppOutput> {
        let mut sri = self.sri.clone();

        // Record SRI for the main JS bundle or entry point
        sri.record_file(
            SriType::ModulePreload, // Or SriType::Script, depending on how it's loaded
            bundle_name,
            SriOptions::default(),
            bundle_path,
        )
        .await?;

        // Handle the optional initializer script
        let initializer_hashed_name = match &self.initializer {
            Some(initializer) => {
                let hashed_name = self.hashed_name(initializer).await?;
                let source = common::strip_prefix(initializer);
                let target = self.cfg.staging_dist.join(&hashed_name);

                self.copy_or_minify_js(source, &target, TopLevelMode::Module)
                    .await?;

                sri.record_file(
                    SriType::ModulePreload,
                    &hashed_name,
                    SriOptions::default(),
                    &target,
                )
                .await?;

                Some(hashed_name)
            }
            None => None,
        };

        let res = PureScriptAppOutput {
            id: self.id,
            cfg: self.cfg.clone(),
            bundle_output: bundle_name.to_string(),
            r#type: self.app_type,
            cross_origin: self.cross_origin,
            integrities: sri,
            initializer: initializer_hashed_name,
            dev_mode_run_script,
        };
        tracing::debug!("{:?}", res);
        Ok(res)
    }
}
