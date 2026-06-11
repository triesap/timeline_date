use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    if std::env::var_os("CARGO_FEATURE_MF2").is_none() {
        return;
    }

    let out_dir = PathBuf::from(std::env::var_os("OUT_DIR").expect("OUT_DIR must be set"));
    let artifact_dir = out_dir.join("timeline_date_i18n");

    let output = mf2_i18n::build::build_project_runtime_artifacts(
        &mf2_i18n::build::ProjectRuntimeBuildOptions::new(
            "i18n/mf2_i18n.toml",
            &artifact_dir,
            "timeline-date-local",
            "1970-01-01T00:00:00Z",
        ),
    )
    .expect("build timeline_date MF2 artifacts");

    for path in output.rerun_if_changed_paths() {
        println!("cargo:rerun-if-changed={}", path.display());
    }

    fs::write(
        out_dir.join("timeline_date_i18n_runtime.rs"),
        render_runtime_module(
            output.default_locale(),
            output.supported_locales(),
            &artifact_dir,
        ),
    )
    .expect("write timeline_date MF2 runtime module");
}

fn render_runtime_module(
    default_locale: &str,
    supported_locales: &[String],
    artifact_dir: &Path,
) -> String {
    let artifact_dir = artifact_dir.display().to_string().replace('\\', "/");
    let supported = supported_locales
        .iter()
        .map(|locale| format!("{locale:?}"))
        .collect::<Vec<_>>()
        .join(", ");
    let packs = supported_locales
        .iter()
        .map(|locale| {
            format!(
                "        mf2_i18n::EmbeddedPack {{ locale: {locale:?}, bytes: include_bytes!(\"{artifact_dir}/packs/{locale}.mf2pack\") }},"
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r#"
pub(crate) const DEFAULT_LOCALE: &str = {default_locale:?};
pub(crate) const SUPPORTED_LOCALES: &[&str] = &[{supported}];

pub(crate) fn runtime() -> Result<&'static mf2_i18n::EmbeddedRuntime, String> {{
    static RUNTIME: std::sync::LazyLock<Result<mf2_i18n::EmbeddedRuntime, String>> =
        std::sync::LazyLock::new(build_runtime);

    match &*RUNTIME {{
        Ok(runtime) => Ok(runtime),
        Err(error) => Err(error.clone()),
    }}
}}

fn build_runtime() -> Result<mf2_i18n::EmbeddedRuntime, String> {{
    let id_map_json = include_bytes!("{artifact_dir}/id-map.json");
    let id_map_hash_text = include_str!("{artifact_dir}/id-map.sha256");
    let id_map = mf2_i18n::IdMap::from_bytes(id_map_json).map_err(|error| error.to_string())?;
    let id_map_hash = mf2_i18n::parse_sha256_literal(id_map_hash_text.trim())
        .map_err(|error| error.to_string())?;
    let id_map_entries = id_map
        .entries()
        .map(|(key, id)| (key.to_owned(), id))
        .collect::<std::collections::BTreeMap<_, _>>();
    let packs: &[mf2_i18n::EmbeddedPack<'static>] = &[
{packs}
    ];

    mf2_i18n::EmbeddedRuntime::new(id_map_entries, id_map_hash, packs, DEFAULT_LOCALE)
        .map_err(|error| error.to_string())
}}
"#
    )
}
