//! Pinned Misskey classification for the unrecovered #9516 diagnostic classes.
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "complete opt-in ecosystem diagnostic evidence"
)]

use super::{native, packets};
use crate::{
    BatchTypeChecker, BatchTypeCheckerOptions, BatchTypeCheckerTrait,
    batch::{source_map::line_col_to_offset, virtual_project::VirtualFile},
    corsa_client::{CorsaProjectClient, LspDiagnostic},
    file_uri::{file_uri_to_path, path_to_file_uri},
};
use serde_json::{Value, json};
use std::{collections::BTreeMap, fs, path::Path, process::Command};
use vize_carton::{String, cstr, line_index::LineBreaks};

const MISSKEY_REVISION: &str = "810faa8e5db4bbf9d6d408a61df148559340ee99";
const CURRENT_TUPLE_READ: &str = "(__vize_args as unknown[])[0]";
const PRIOR_TUPLE_READ: &str = "__vize_args[0]";
const CURRENT_FALLTHROUGH: &str = "{ [K in `data-${string}` | `data${Capitalize<string>}`]?: unknown } & { [K in `aria-${string}` | `aria${Capitalize<string>}`]?: unknown }";
const PRIOR_FALLTHROUGH: &str =
    "{ [K in `data-${string}`]?: unknown } & { [K in `aria-${string}`]?: unknown }";

#[test]
fn pinned_misskey_separates_tuple_and_property_generated_diagnostic_causes() {
    if std::env::var("VIZE_TEST_MISSKEY_GENERATED_DIAGNOSTICS_9516").as_deref() != Ok("1") {
        return;
    }
    let Some(native) = native() else { return };
    let repository = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let fixture = repository.join("tests/_fixtures/_git/misskey");
    let frontend = fixture.join("packages/frontend").canonicalize().unwrap();
    let revision = command_stdout(&fixture, &["rev-parse", "HEAD"]);
    assert_eq!(
        revision, MISSKEY_REVISION,
        "the registry pin is the authority"
    );

    let mut checker = BatchTypeChecker::with_options_and_corsa_path(
        &frontend,
        BatchTypeCheckerOptions {
            tsconfig_path: Some(frontend.join("tsconfig.json")),
            ..Default::default()
        },
        Some(&native),
    )
    .unwrap();
    checker.set_server_count(Some(1));
    checker.set_checker_count(Some(1));
    checker.scan_project().unwrap();
    let assembled = checker.check_project().unwrap();
    let (mirror, config) = checker.native_project_paths_for_test();
    let files = checker.virtual_files();
    let requested = files
        .iter()
        .filter(|file| {
            file.original_path
                .extension()
                .and_then(|value| value.to_str())
                == Some("vue")
        })
        .map(|file| path_to_file_uri(&file.virtual_path))
        .collect::<Vec<_>>();
    assert!(
        !requested.is_empty(),
        "the pinned project must contain Vue inputs"
    );

    let custody = json!({
        "scope":"new reproduction on the pinned Misskey source; not the unavailable historical 415-finding packet",
        "issue":9516,"fixtureRepository":"https://github.com/misskey-dev/misskey",
        "fixtureRevision":revision,"fixtureRoot":fixture,"frontendRoot":frontend,
        "sourceRevision":std::env::var("GITHUB_SHA").ok(),
        "nativeBinary":native,"generatedMirror":mirror,"generatedConfig":config,
        "wholeGeneratedConfig":fs::read_to_string(&config).unwrap(),
        "requestedVueDocuments":requested.len(),
        "wholeAssembledResult":cstr!("{assembled:#?}"),
    });
    packets::retain("misskey-9516-custody", &custody);

    let current = diagnose(
        &native, &mirror, &config, &files, &requested, "current", None,
    );
    let current_property = target_keys(&current.raw, 2353);

    let tuple_mutations = files
        .iter()
        .filter(|file| file.content.contains(CURRENT_TUPLE_READ))
        .map(|file| {
            (
                file.virtual_path.clone(),
                file.content
                    .replace(CURRENT_TUPLE_READ, PRIOR_TUPLE_READ)
                    .into_bytes(),
            )
        })
        .collect::<Vec<_>>();
    assert!(
        !tuple_mutations.is_empty(),
        "Misskey must exercise component event extraction"
    );
    let tuple = {
        let _restore = RestoredFiles::write(tuple_mutations);
        diagnose(
            &native,
            &mirror,
            &config,
            &files,
            &requested,
            "pre-9435-tuple-expression",
            Some(ReplayCause::Tuple),
        )
    };
    let tuple_rows = tuple
        .evidence
        .iter()
        .filter(|row| {
            row["diagnostic"]["code"] == 2493 && row["replayExpressionOwner"] == "generated"
        })
        .collect::<Vec<_>>();
    assert!(
        !tuple_rows.is_empty(),
        "the exact pre-9435 tuple expression must reproduce generated TS2493"
    );

    let helper = config.with_file_name(crate::virtual_ts::SHARED_PREAMBLE_FILE_NAME);
    let helper_current = fs::read_to_string(&helper).unwrap();
    assert_eq!(helper_current.matches(CURRENT_FALLTHROUGH).count(), 1);
    let property = {
        let _restore = RestoredFiles::write(vec![(
            helper.clone(),
            helper_current
                .replacen(CURRENT_FALLTHROUGH, PRIOR_FALLTHROUGH, 1)
                .into_bytes(),
        )]);
        diagnose(
            &native,
            &mirror,
            &config,
            &files,
            &requested,
            "pre-9437-data-aria-contract",
            Some(ReplayCause::Property),
        )
    };
    let property_rows = property
        .evidence
        .iter()
        .filter(|row| {
            row["diagnostic"]["code"] == 2353
                && !current_property.contains(&diagnostic_key(&row["uri"], &row["diagnostic"]))
        })
        .collect::<Vec<_>>();
    assert!(
        !property_rows.is_empty(),
        "the exact pre-9437 data/aria contract must reproduce new TS2353"
    );
    assert!(
        property_rows
            .iter()
            .all(|row| row["authoredOwner"].is_null()),
        "the reproduced property findings must remain generated-owned: {property_rows:#?}"
    );
    let current_ts2493 = current
        .evidence
        .iter()
        .filter(|row| row["diagnostic"]["code"] == 2493)
        .count();
    let current_ts2353 = current
        .evidence
        .iter()
        .filter(|row| row["diagnostic"]["code"] == 2353)
        .count();
    assert_eq!(
        (current_ts2493, current_ts2353),
        (0, 0),
        "current generation must leave neither target diagnostic in inference-only code: {:#?}",
        current.evidence
    );
    println!(
        "Misskey #9516 new reference: current TS2493={current_ts2493}, current TS2353={current_ts2353}, replay generated TS2493={}, replay new TS2353={}",
        tuple_rows.len(),
        property_rows.len()
    );

    packets::retain(
        "misskey-9516-classification",
        &json!({
            "custody":custody,
            "baseline":current.packet,
            "tupleCause":{
                "historicalFix":"2e5e976ec6105311fd7a4399c0f6cd8b6dfb8250",
                "inverse":{"current":CURRENT_TUPLE_READ,"replay":PRIOR_TUPLE_READ},
                "reproducedGeneratedTs2493":tuple_rows.len(),
                "packet":tuple.packet,
            },
            "propertyCause":{
                "historicalFix":"96f6a1b52b",
                "inverse":{"current":CURRENT_FALLTHROUGH,"replay":PRIOR_FALLTHROUGH},
                "reproducedNewTs2353":property_rows.len(),
                "packet":property.packet,
            },
            "inferenceProbeCause":{
                "sourceRun":39425591681_u64,
                "sourceArtifact":"real-project-matrix-12/misskey-generated-diagnostics-9516",
                "observedGeneratedTs2353":173,
                "observedByAttribute":{"readonly":132,"scroll":12,"max":12,"canClose":6,"type":4,"expanded":4,"foldable":2,"withPasswordToggle":1},
                "ownership":{"slotPayloadProbe":172,"genericTemplateRefProbe":1,"authoredMappings":0},
                "fix":"open only duplicated inference probes; mapped component checks remain authoritative",
            },
            "classification":"three independent generated-code causes across the two diagnostic classes; current generation repairs all without diagnostic suppression or checker give-up changes",
        }),
    );
}

struct Phase {
    raw: Vec<(String, Vec<LspDiagnostic>)>,
    evidence: Vec<Value>,
    packet: Value,
}

#[derive(Clone, Copy)]
enum ReplayCause {
    Tuple,
    Property,
}

fn diagnose(
    native: &Path,
    mirror: &Path,
    config: &Path,
    files: &[&VirtualFile],
    requested: &[String],
    phase: &str,
    replay: Option<ReplayCause>,
) -> Phase {
    let mut client = CorsaProjectClient::new_for_workspace_config(
        Some(native.to_str().unwrap()),
        mirror,
        config,
    )
    .unwrap();
    client.begin_batch_test_receipt(true);
    let raw = client.request_diagnostics_batch(requested).unwrap();
    let authority = client.batch_test_receipt(requested.first().unwrap());
    client.shutdown().unwrap();
    let evidence = evidence(files, &raw, replay);
    let packet = json!({
        "phase":phase,"nativeAuthority":authority,"requestedUris":requested,
        "wholeRawDiagnosticVector":raw,"targetEvidence":evidence,
        "wholeGeneratedConfig":fs::read_to_string(config).unwrap(),
    });
    packets::retain(&cstr!("misskey-9516-{phase}"), &packet);
    Phase {
        raw,
        evidence,
        packet,
    }
}

fn evidence(
    files: &[&VirtualFile],
    raw: &[(String, Vec<LspDiagnostic>)],
    replay: Option<ReplayCause>,
) -> Vec<Value> {
    let by_path = files
        .iter()
        .map(|file| (file.virtual_path.clone(), *file))
        .collect::<BTreeMap<_, _>>();
    raw.iter()
        .flat_map(|(uri, diagnostics)| {
            let path = file_uri_to_path(uri).unwrap();
            let file = by_path.get(&path).unwrap();
            let replay_code = fs::read_to_string(&path).unwrap();
            diagnostics
                .iter()
                .filter(|diagnostic| matches!(code(diagnostic), Some(2493 | 2353)))
                .map(move |diagnostic| {
                    let offset = line_col_to_offset(
                        &replay_code,
                        diagnostic.range.start.line,
                        diagnostic.range.start.character,
                    )
                    .unwrap();
                    let (current_offset, replay_owner) = match replay {
                        Some(ReplayCause::Tuple) => tuple_current_offset(
                            file.content.as_str(),
                            &replay_code,
                            usize::try_from(offset).unwrap(),
                        ),
                        Some(ReplayCause::Property) | None => {
                            (usize::try_from(offset).unwrap(), None)
                        }
                    };
                    let authored = file
                        .source_map
                        .get_original_position(u32::try_from(current_offset).unwrap())
                        .map(|(authored_offset, source_column, block)| {
                            let source = fs::read_to_string(&file.original_path).unwrap();
                            let position = LineBreaks::Lsp.offset_to_position(
                                &source,
                                usize::try_from(authored_offset).unwrap(),
                            );
                            json!({
                                "path":file.original_path,"offset":authored_offset,
                                "line":position.0,"character":position.1,
                                "sourceColumn":source_column,"block":format!("{block:?}"),
                            })
                        });
                    json!({
                        "uri":uri,"diagnostic":diagnostic,
                        "generatedOffset":offset,"currentEquivalentOffset":current_offset,
                        "replayExpressionOwner":replay_owner,
                        "authoredOwner":authored,
                        "completeReplayCode":packets::blob(replay_code.as_bytes()),
                        "completeCurrentCode":packets::blob(file.content.as_bytes()),
                        "completeAuthoredSource":packets::blob(&fs::read(&file.original_path).unwrap()),
                        "completeSourceMap":packets::blob(cstr!("{:#?}",file.source_map).as_bytes()),
                    })
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

fn tuple_current_offset(
    current: &str,
    replay: &str,
    offset: usize,
) -> (usize, Option<&'static str>) {
    let current_occurrences = current
        .match_indices(CURRENT_TUPLE_READ)
        .collect::<Vec<_>>();
    assert_eq!(
        current.replace(CURRENT_TUPLE_READ, PRIOR_TUPLE_READ),
        replay
    );
    let mut replay_cursor = 0;
    let mut current_cursor = 0;
    let replacements = current_occurrences
        .iter()
        .map(|(current_start, _)| {
            replay_cursor += current_start - current_cursor;
            let replay_start = replay_cursor;
            replay_cursor += PRIOR_TUPLE_READ.len();
            current_cursor = current_start + CURRENT_TUPLE_READ.len();
            (replay_start, *current_start)
        })
        .collect::<Vec<_>>();
    for (replay_start, current_start) in &replacements {
        let replay_index = replay_start + "__vize_args[".len();
        if offset >= *replay_start && offset < replay_start + PRIOR_TUPLE_READ.len() {
            let current_index = current_start + "(__vize_args as unknown[])[".len();
            let translated = if offset >= replay_index {
                current_index + (offset - replay_index)
            } else {
                current_start + "(".len() + (offset - replay_start)
            };
            return (translated, Some("generated"));
        }
    }
    let preceding = replacements
        .iter()
        .filter(|(replay_start, _)| replay_start + PRIOR_TUPLE_READ.len() <= offset)
        .count();
    (
        offset + preceding * (CURRENT_TUPLE_READ.len() - PRIOR_TUPLE_READ.len()),
        None,
    )
}

#[test]
fn tuple_replay_mapping_ignores_preexisting_native_event_reads() {
    let current = cstr!("native {PRIOR_TUPLE_READ}; component {CURRENT_TUPLE_READ}; after");
    let replay = current.replace(CURRENT_TUPLE_READ, PRIOR_TUPLE_READ);
    let replay_component = replay.rfind(PRIOR_TUPLE_READ).unwrap();
    let replay_index = replay_component + "__vize_args[".len();
    let current_component = current.find(CURRENT_TUPLE_READ).unwrap();
    let current_index = current_component + "(__vize_args as unknown[])[".len();
    assert_eq!(
        tuple_current_offset(&current, &replay, replay_index),
        (current_index, Some("generated"))
    );
    assert_eq!(
        tuple_current_offset(&current, &replay, replay.len()),
        (current.len(), None,)
    );
}

fn code(diagnostic: &LspDiagnostic) -> Option<u64> {
    diagnostic
        .code
        .as_ref()
        .and_then(|value| value.as_u64().or_else(|| value.as_str()?.parse().ok()))
}

fn target_keys(raw: &[(String, Vec<LspDiagnostic>)], target: u64) -> Vec<String> {
    raw.iter()
        .flat_map(|(uri, diagnostics)| {
            diagnostics
                .iter()
                .filter(move |diagnostic| code(diagnostic) == Some(target))
                .map(move |diagnostic| {
                    diagnostic_key(&json!(uri), &serde_json::to_value(diagnostic).unwrap())
                })
        })
        .collect()
}

fn diagnostic_key(uri: &Value, diagnostic: &Value) -> String {
    serde_json::to_string(&(uri, diagnostic)).unwrap().into()
}

fn command_stdout(cwd: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .unwrap();
    assert!(output.status.success(), "git failed: {output:#?}");
    String::from_utf8(output.stdout).unwrap().trim().into()
}

struct RestoredFiles(Vec<(std::path::PathBuf, Vec<u8>)>);

impl RestoredFiles {
    fn write(replacements: Vec<(std::path::PathBuf, Vec<u8>)>) -> Self {
        let originals = replacements
            .iter()
            .map(|(path, _)| (path.clone(), fs::read(path).unwrap()))
            .collect::<Vec<_>>();
        for (path, bytes) in replacements {
            fs::write(path, bytes).unwrap();
        }
        Self(originals)
    }
}

impl Drop for RestoredFiles {
    fn drop(&mut self) {
        for (path, bytes) in &self.0 {
            fs::write(path, bytes).unwrap();
        }
    }
}
