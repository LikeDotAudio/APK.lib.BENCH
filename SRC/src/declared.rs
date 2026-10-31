// Part of the APK.audio project — http://APK.audio — made by Anthony Kuzub
// MIT Licence. Free, for everyone, for ever. Full text in LICENSE at the root.
//! Instruments declared in their own capability sheet rather than found.
//!
//! Discovery asks `*IDN?` and believes the answer. That is the right test for
//! almost everything, and useless for a switch matrix: the HP 3235 takes
//! commands and answers none of them, so a probe concludes it is absent while
//! it sits there routing audio. A scan can only ever find instruments that
//! talk. So the sheet says so, next to everything else true of the model:
//!
//! ```json
//! "declared": {
//!   "resource": "TCPIP::44.44.44.222::gpib7,10::INSTR",
//!   "listen_only": true,
//!   "on_connect": ["Open_All"]
//! }
//! ```
//!
//! THE LIBRARY, NOT THE REPOSITORY, IS THE ARGUMENT. Every function here takes
//! the instrument library directory itself (`PODS/POD.THICK/INSTRUMENTS` on a
//! checkout, `/opt/apk/Instruments` in the VISA plugin's image,
//! `/app/Instruments` on the node). Finding it is each process's business; the
//! orchestrator resolves it with `library_root`, the plugin from its env.
//!
//! Moved here from the orchestrator's `instruments/mod.rs` (PLAN-3316.01 step
//! 6.6): the VISA scan reads the declared list, the panel builder reads every
//! sheet, and they are now two processes.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde_json::Value;

/// An instrument that is wired to the bench but cannot be discovered.
#[derive(Debug, Clone)]
pub struct Declared {
    pub dtype: String,
    pub model: String,
    pub manufacturer: String,
    pub notes: String,
    pub resource: String,
    pub listen_only: bool,
    /// SCPI to send the moment this instrument is declared, already resolved
    /// from the model's own vocabulary.
    pub on_connect: Vec<String>,
}

/// The key a model directory is known by: `HP_8903B` -> `8903B`; a bare
/// `34401A` is unchanged.
fn model_key(dir_name: &str) -> &str {
    dir_name.splitn(2, '_').last().unwrap_or(dir_name)
}

/// Every model's capability sheet: `(model key, model directory, sheet)`, from
/// `<library>/INSTRUMENT.<Family>/<Model>/<Model>.gui` (a bare `<Family>/` reads the
/// same: the folder name is never the type, the sheet's `type` is). Sorted, and the FIRST sheet for a
/// key wins, so two benches serving the same commit read the same answer.
///
/// `<Model>.gui` is located by extension: the folder may carry a manufacturer
/// prefix (`HP_8903B`) or an underscore in the model itself (`Porta_one`), so
/// deriving the filename from the directory would be wrong in both directions.
pub fn sheets(library: &Path) -> Vec<(String, PathBuf, Value)> {
    let mut out: Vec<(String, PathBuf, Value)> = Vec::new();
    let mut seen = std::collections::HashSet::new();
    let Ok(families) = std::fs::read_dir(library) else {
        return out;
    };
    let mut families: Vec<_> = families.flatten().map(|e| e.path()).collect();
    families.sort();
    for family in families {
        let Ok(models) = std::fs::read_dir(&family) else {
            continue;
        };
        let mut models: Vec<_> = models.flatten().map(|e| e.path()).collect();
        models.sort();
        for model_directory in models {
            if !model_directory.is_dir()
                || model_directory.file_name().and_then(|n| n.to_str()) == Some("gui")
            {
                continue;
            }
            let Some(path) = std::fs::read_dir(&model_directory).ok().and_then(|rd| {
                let mut hits: Vec<PathBuf> = rd
                    .flatten()
                    .map(|e| e.path())
                    .filter(|p| p.extension().map_or(false, |e| e == "gui"))
                    .collect();
                hits.sort();
                hits.into_iter().next()
            }) else {
                continue;
            };
            let Some(dir_name) = model_directory.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            let key = model_key(dir_name).to_string();
            if seen.contains(&key) {
                continue;
            }
            if let Ok(body) = std::fs::read_to_string(&path) {
                if let Ok(value) = serde_json::from_str::<Value>(&body) {
                    seen.insert(key.clone());
                    out.push((key, model_directory, value));
                }
            }
        }
    }
    out
}

/// Where a model's two authored files live, by model name.
pub fn model_directory(library: &Path, model: &str) -> Option<PathBuf> {
    let families = std::fs::read_dir(library).ok()?;
    for family in families.flatten() {
        let Ok(models) = std::fs::read_dir(family.path()) else { continue };
        for directory in models.flatten() {
            let name = directory.file_name();
            let name = name.to_str().unwrap_or_default();
            if name == "gui" {
                continue;
            }
            if model_key(name) == model {
                return Some(directory.path());
            }
        }
    }
    None
}

/// One command's SCPI out of a model's KOMAND table, whichever verb holds it.
pub fn komand_scpi(library: &Path, model: &str, command: &str) -> Option<String> {
    let directory = model_directory(library, model)?;
    let path = directory.join(format!("{model}.komand"));
    let document: Value = serde_json::from_str(&std::fs::read_to_string(path).ok()?).ok()?;
    for verb in ["do", "set", "rig", "nab"] {
        if let Some(s) = document
            .get(verb)
            .and_then(|v| v.get(command))
            .and_then(|c| c.get("scpi"))
            .and_then(|s| s.as_str())
        {
            return Some(s.to_string());
        }
    }
    None
}

/// Every declared instrument in `library`, in a stable order so two runs stamp
/// the same `Dev` index on the same box.
pub fn declared_devices(library: &Path) -> Vec<Declared> {
    let sheets: HashMap<String, Value> =
        sheets(library).into_iter().map(|(key, _, sheet)| (key, sheet)).collect();
    let mut out = Vec::new();
    for (model, caps) in sheets {
        let Some(d) = caps.get("declared") else { continue };
        let Some(resource) = d.get("resource").and_then(|r| r.as_str()) else {
            println!("[instrument-gui] {model} declares itself with no resource — skipped");
            continue;
        };
        let Some(dtype) = caps.get("type").and_then(|t| t.as_str()) else {
            println!("[instrument-gui] {model} is declared but names no type — skipped");
            continue;
        };
        // `on_connect` names COMMANDS, not SCPI: an installation fact belongs in
        // the sheet, but the words an instrument understands belong in its
        // vocabulary, and writing them twice is how the two drift apart.
        let on_connect = d
            .get("on_connect")
            .and_then(|v| v.as_array())
            .map(|names| {
                names
                    .iter()
                    .filter_map(|n| n.as_str())
                    .filter_map(|name| match komand_scpi(library, &model, name) {
                        Some(scpi) => Some(scpi),
                        None => {
                            println!(
                                "[instrument-gui] {model} names '{name}' on connect, \
                                 and its KOMAND table has no such command — skipped"
                            );
                            None
                        }
                    })
                    .collect()
            })
            .unwrap_or_default();

        out.push(Declared {
            dtype: dtype.to_string(),
            manufacturer: caps.get("manufacturer").and_then(|m| m.as_str()).unwrap_or("").to_string(),
            notes: caps.get("notes").and_then(|n| n.as_str()).unwrap_or("").to_string(),
            resource: resource.to_string(),
            listen_only: d.get("listen_only") == Some(&Value::Bool(true)),
            on_connect,
            model,
        });
    }
    out.sort_by(|a, b| (&a.dtype, &a.model).cmp(&(&b.dtype, &b.model)));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A minimal library checked in beside this crate (`SRC/fixtures/library`),
    /// so the sheet-format test runs in a standalone clone too. It used to read
    /// `CARGO_MANIFEST_DIR/../../../INSTRUMENTS` and skip silently when that
    /// sibling repository was absent (2026-10-30 audit, finding 5).
    fn fixture() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/library")
    }

    #[test]
    fn the_fixture_library_declares_the_switch_matrix() {
        let library = fixture();
        let found = declared_devices(&library);
        assert_eq!(found.len(), 1, "only the 3235 has a `declared` block: {found:?}");
        let matrix = &found[0];
        assert_eq!((matrix.dtype.as_str(), matrix.model.as_str()), ("Router", "3235"));
        assert!(matrix.resource.contains("gpib"), "{}", matrix.resource);
        assert!(matrix.listen_only);
        // `on_connect` names commands; the KOMAND table resolves them, and a
        // name it does not hold is skipped rather than sent as text.
        assert_eq!(matrix.on_connect, vec!["OPEN 000-999".to_string()]);
        // The manufacturer prefix on the folder is not the model key.
        assert_eq!(model_directory(&library, "3235").unwrap().file_name().unwrap(), "HP_3235");
        let keys: Vec<String> = sheets(&library).into_iter().map(|(k, _, _)| k).collect();
        assert_eq!(keys, vec!["34401A".to_string(), "3235".to_string()]);
    }

    #[test]
    fn an_empty_library_declares_nothing() {
        let empty = std::env::temp_dir().join(format!("apk-bench-empty-{}", std::process::id()));
        std::fs::create_dir_all(&empty).unwrap();
        assert!(declared_devices(&empty).is_empty());
        let _ = std::fs::remove_dir_all(&empty);
    }
}
