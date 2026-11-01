//! Real cold parameter mentions versus a fully materialized same-session owner.
#![expect(clippy::disallowed_macros, reason = "complete native failure evidence")]

use super::{TypeProbe, probe_types_at_positions};
use corsa::{
    api::{ApiMode, ApiSpawnConfig, ProjectSession, TypeProbeOptions},
    runtime::block_on,
};
use serde_json::{Value, json};
use std::{fs, path::Path};

macro_rules! fixture {
    ($name:literal) => {
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/_fixtures/differential/linter/symbol-owner-probes-11267/",
            $name
        ))
    };
}

struct Case {
    id: &'static str,
    source: &'static str,
    names: &'static [&'static [&'static str]],
}
const CASES: &[Case] = &[
    Case {
        id: "required",
        source: fixture!("Required.ts.txt"),
        names: &[&["value", "label"]],
    },
    Case {
        id: "optional",
        source: fixture!("Optional.ts.txt"),
        names: &[&["value", "label"]],
    },
    Case {
        id: "rest",
        source: fixture!("Rest.ts.txt"),
        names: &[&["values"]],
    },
    Case {
        id: "this",
        source: fixture!("This.ts.txt"),
        names: &[&["value"]],
    },
    Case {
        id: "overloads",
        source: fixture!("Overloads.ts.txt"),
        names: &[&["value"], &["value"]],
    },
    Case {
        id: "generic",
        source: fixture!("Generic.ts.txt"),
        names: &[&["value"]],
    },
    Case {
        id: "zero",
        source: fixture!("Zero.ts.txt"),
        names: &[&[]],
    },
];

fn probe_json(probe: &TypeProbe) -> Value {
    json!({"typeTexts":probe.type_texts,"propertyNames":probe.property_names,
        "propertyTypes":probe.property_types,"callSignatures":probe.call_signatures,
        "returnTypes":probe.return_types})
}

fn probe_result(result: corsa::Result<Vec<Option<TypeProbe>>>) -> Value {
    match result {
        Ok(probes) => json!({"state":"complete", "probes":probes.iter()
            .map(|probe|probe.as_ref().map(probe_json)).collect::<Vec<_>>()}),
        Err(error) => json!({"state":"error", "error":format!("{error}"),
            "errorDebug":format!("{error:?}")}),
    }
}

async fn inspect(session: &ProjectSession, file: &str, position: u32) -> Value {
    let options = || {
        vec![TypeProbeOptions {
            load_property_types: false,
            load_signatures: true,
        }]
    };
    // This must be first: no parameter endpoint or full parameter reference has
    // primed this connection. A non-reproduction remains visible in the result.
    let cold =
        probe_result(probe_types_at_positions(session, file, vec![position], options()).await);
    let mut observation = json!({"cold":cold,"materialized":[],
        "snapshot":session.snapshot().handle,"project":session.project_handle()});
    let materialization = async {
        let response = session
            .get_type_at_position(file, position)
            .await?
            .ok_or_else(|| corsa::CorsaError::Protocol("missing authored probe type".into()))?;
        let signatures = session
            .get_signatures_of_type(response.id.clone(), 0)
            .await?;
        observation["type"] = json!(response);
        observation["signatures"] = json!(signatures);
        for signature in signatures {
            // Existing public endpoint: same client, project, snapshot and exact
            // signature, not a new session, source-name lookup or handle rewrite.
            let parameters = session
                .client()
                .get_parameters_of_signature(
                    session.snapshot().handle.clone(),
                    session.project_handle(),
                    signature.id.clone(),
                )
                .await?;
            let index = observation["materialized"].as_array().unwrap().len();
            observation["materialized"]
                .as_array_mut()
                .unwrap()
                .push(json!({"signature":signature,"parameters":parameters}));
            let types = session
                .get_types_of_symbols(parameters.iter().map(|p| p.id.clone()).collect())
                .await?;
            observation["materialized"][index]["types"] = json!(types);
            let mut rendered = Vec::new();
            for response in types.iter().flatten() {
                rendered.push(session.render_type_texts(response).await?);
            }
            observation["materialized"][index]["renderedTypes"] = json!(rendered);
            // This parameter is deliberately outside the ordinary parameter list.
            // Retain its full symbol/type without adding it to Patina's argument list.
            let this = session
                .client()
                .get_this_parameter_of_signature(
                    session.snapshot().handle.clone(),
                    session.project_handle(),
                    signature.id.clone(),
                )
                .await?;
            observation["materialized"][index]["thisParameter"] = json!(this);
            if let Some(this) = this {
                observation["materialized"][index]["thisType"] =
                    json!(session.get_type_of_symbol(this.id).await?);
            }
        }
        Ok::<_, corsa::CorsaError>(())
    }
    .await;
    observation["materialization"] = match materialization {
        Ok(()) => json!({"state":"complete"}),
        Err(error) => {
            json!({"state":"error","error":format!("{error}"),"errorDebug":format!("{error:?}")})
        }
    };
    let warm =
        probe_result(probe_types_at_positions(session, file, vec![position], options()).await);
    observation["warm"] = warm;
    observation
}

fn observe(runtime: &Path, case: &Case, imported: bool) -> Value {
    let owner = tempfile::tempdir().unwrap();
    let root = vize_l0::path::canonicalize_non_verbatim(owner.path());
    fs::write(root.join("package.json"), fixture!("package.json.txt")).unwrap();
    fs::write(root.join("tsconfig.json"), fixture!("tsconfig.json.txt")).unwrap();
    fs::write(root.join("dependency.ts"), case.source).unwrap();
    let source = if imported {
        fixture!("Probe.ts.txt").to_owned()
    } else {
        format!(
            "{}\n{}",
            case.source,
            fixture!("Probe.ts.txt")
                .strip_prefix("import { check } from \"./dependency\";\n")
                .unwrap()
        )
    };
    let main = root.join("main.ts");
    fs::write(&main, &source).unwrap();
    let file = main.to_string_lossy().replace('\\', "/");
    let position = source[..source.find("probe =").unwrap()]
        .encode_utf16()
        .count() as u32;
    let mode = if vize_l0::corsa_api_mode::uses_async_json_rpc_api(runtime) {
        ApiMode::AsyncJsonRpcStdio
    } else {
        ApiMode::SyncMsgpackStdio
    };
    let observed = block_on(async {
        let session = ProjectSession::spawn(
            ApiSpawnConfig::new(runtime.to_path_buf())
                .with_mode(mode)
                .with_cwd(&root),
            root.join("tsconfig.json").to_string_lossy(),
            Some(file.as_str().into()),
        )
        .await?;
        let result = inspect(&session, &file, position).await;
        let client = session.client().clone();
        drop(session);
        let closed = client.close().await;
        Ok::<_, corsa::CorsaError>(json!({"result":result,
            "closed":match closed { Ok(())=>json!({"state":"closed"}),
                Err(error)=>json!({"state":"error","error":format!("{error}"),
                    "errorDebug":format!("{error:?}")}) }}))
    });
    json!({"case":case.id,"imported":imported,"main":source,"dependency":case.source,
        "file":file,"position":position,"apiMode":format!("{mode:?}"),
        "expectedNames":case.names,"observation":match observed { Ok(value)=>value,
            Err(error)=>json!({"setupError":format!("{error}"),"errorDebug":format!("{error:?}")}) }})
}

#[test]
fn cold_and_materialized_signature_probes_stay_identical() {
    if std::env::var_os("VIZE_TEST_DISABLE_TSGO").is_some() {
        return;
    }
    let Some(runtime) = std::env::var_os("CORSA_PATH") else {
        assert!(
            std::env::var_os("VIZE_TEST_REQUIRE_TSGO").is_none(),
            "the mandatory owner test requires an explicit selected runtime"
        );
        return;
    };
    let observations = CASES
        .iter()
        .flat_map(|case| {
            [
                observe(Path::new(&runtime), case, true),
                observe(Path::new(&runtime), case, false),
            ]
        })
        .collect::<Vec<_>>();
    let capture = json!({"schema":"vize.patina.symbol-owner-probes.v1",
        "scope":"causal source reduction, not the unavailable original 1027-file app",
        "selectedRuntime":runtime.to_string_lossy(),"observations":observations});
    let raw = serde_json::to_string_pretty(&capture).unwrap() + "\n";
    eprintln!("11267 complete owner observations: {raw}");
    if let Some(path) = std::env::var_os("VIZE_SYMBOL_OWNER_CAPTURE") {
        fs::write(path, raw).unwrap();
    }
    for row in &observations {
        assert_eq!(
            row["observation"]["closed"],
            json!({"state":"closed"}),
            "{row:#}"
        );
        let result = &row["observation"]["result"];
        assert_eq!(
            result["materialization"],
            json!({"state":"complete"}),
            "{row:#}"
        );
        assert_eq!(result["warm"]["state"], "complete", "{row:#}");
        assert_eq!(
            result["warm"]["probes"].as_array().unwrap().len(),
            1,
            "{row:#}"
        );
        assert!(result["warm"]["probes"][0].is_object(), "{row:#}");
        let materialized = result["materialized"].as_array().unwrap();
        assert_eq!(
            json!(
                materialized
                    .iter()
                    .map(|part| part["parameters"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|p| p["name"].clone())
                        .collect::<Vec<_>>())
                    .collect::<Vec<_>>()
            ),
            row["expectedNames"],
            "{row:#}"
        );
        for part in materialized {
            assert_eq!(
                json!(
                    part["parameters"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|p| p["id"].clone())
                        .collect::<Vec<_>>()
                ),
                part["signature"]["parameters"],
                "{row:#}"
            );
            assert_eq!(
                part["types"].as_array().unwrap().len(),
                part["parameters"].as_array().unwrap().len()
            );
            assert_eq!(
                part["types"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(Value::is_null)
                    .collect::<Vec<_>>(),
                vec![false; part["parameters"].as_array().unwrap().len()],
                "{row:#}"
            );
            assert_eq!(
                part["thisParameter"].is_object(),
                row["case"] == "this",
                "{row:#}"
            );
        }
        assert_eq!(
            result["warm"]["probes"][0]["callSignatures"],
            json!(
                materialized
                    .iter()
                    .map(|part| part["renderedTypes"].clone())
                    .collect::<Vec<_>>()
            ),
            "{row:#}"
        );
        assert_eq!(result["cold"], result["warm"], "{row:#}");
    }
}
