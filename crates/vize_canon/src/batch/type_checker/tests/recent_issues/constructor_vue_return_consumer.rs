//! Separate opt-in source carrier; the old three-test filter stays unchanged.

use crate::batch::{VirtualProject, virtual_project::MaterializedSourceMappingKind};
use crate::corsa_bridge::constructor_vue_proof::{
    Anchor, SourceExpectation, UsageRequest, capture,
};
use corsa::{
    CorsaError,
    api::{ManagedSnapshot, ProjectSession},
    runtime::block_on,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fmt, fs,
    io::Write,
    path::{Path, PathBuf},
};
use vize_l0::{String, cstr};

const OUTPUT: &str = "VIZE_CONSTRUCTOR_VUE_PROOF_OUTPUT";
const FILE: &str = "constructor-vue-return.json";

pub(super) struct Input<'a> {
    pub(super) project: &'a VirtualProject,
    pub(super) root: &'a Path,
    pub(super) config: &'a Path,
    pub(super) generated_file: &'a Path,
    pub(super) generated_text: &'a str,
    pub(super) authored_text: &'a str,
    pub(super) component_text: &'a str,
}

#[derive(Debug)]
pub(super) enum Failure {
    Input(&'static str),
    Io(std::io::Error),
    Json(serde_json::Error),
    // Preserve the original SDK error, even if evidence persistence also fails.
    Native(CorsaError, Option<Box<Failure>>),
}

impl fmt::Display for Failure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Input(message) => formatter.write_str(message),
            Self::Io(error) => write!(formatter, "{error}"),
            Self::Json(error) => write!(formatter, "{error}"),
            Self::Native(error, persistence) => {
                write!(formatter, "{error}; evidence: {persistence:?}")
            }
        }
    }
}

fn output_path(
    raw: Option<std::ffi::OsString>,
    evidence: &Path,
) -> Result<Option<PathBuf>, Failure> {
    let Some(raw) = raw else { return Ok(None) };
    let path = PathBuf::from(raw);
    if path != evidence.join(FILE) || !path.is_absolute() {
        return Err(Failure::Input(
            "sidecar output must be the exact existing evidence child",
        ));
    }
    Ok(Some(path))
}

fn name(path: &Path) -> Result<String, Failure> {
    path.to_str()
        .filter(|_| path.is_absolute())
        .map(String::from)
        .ok_or(Failure::Input(
            "source identity must be an absolute UTF8 path",
        ))
}

fn anchor(source: &str, context: &str, token: &str) -> Result<Anchor, Failure> {
    let mut contexts = source.match_indices(context);
    let Some((offset, _)) = contexts.next() else {
        return Err(Failure::Input(
            "request anchor context is absent or ambiguous",
        ));
    };
    if contexts.next().is_some() {
        return Err(Failure::Input(
            "request anchor context is absent or ambiguous",
        ));
    }
    let mut tokens = context.match_indices(token);
    let Some((inside, _)) = tokens.next() else {
        return Err(Failure::Input(
            "request anchor token is absent or ambiguous",
        ));
    };
    if tokens.next().is_some() || token.is_empty() {
        return Err(Failure::Input(
            "request anchor token is absent or ambiguous",
        ));
    }
    let prefix = source
        .get(..offset + inside)
        .ok_or(Failure::Input("invalid UTF8 anchor boundary"))?;
    let start = u32::try_from(prefix.encode_utf16().count())
        .map_err(|_| Failure::Input("anchor exceeds UTF16 range"))?;
    let length = u32::try_from(token.encode_utf16().count())
        .map_err(|_| Failure::Input("anchor exceeds UTF16 range"))?;
    let end = start
        .checked_add(length)
        .ok_or(Failure::Input("anchor exceeds UTF16 range"))?;
    Ok(Anchor {
        start,
        end,
        position: start,
        text: token.into(),
    })
}

type Member<'a> = (&'a Path, &'a Path, &'a str, &'a str, bool);

fn component_source<'a>(
    members: impl IntoIterator<Item = Member<'a>>,
    original: &Path,
    expected: &str,
) -> Result<SourceExpectation, Failure> {
    let mut selected = members.into_iter().filter(|member| member.0 == original);
    let Some((_, materialized, authored, code, synthetic)) = selected.next() else {
        return Err(Failure::Input(
            "component must have one exact owned materialized source",
        ));
    };
    if selected.next().is_some() || synthetic || authored != expected {
        return Err(Failure::Input(
            "component must have one exact owned materialized source",
        ));
    }
    let file = name(materialized)?;
    Ok(SourceExpectation {
        path: file.clone(),
        file,
        text: code.into(),
    })
}

fn request(input: &Input<'_>) -> Result<UsageRequest, Failure> {
    let documents = input.project.materialized_source_documents();
    let target = vize_carton::path::canonicalize_non_verbatim(&input.root.join("component.ts"));
    let component = component_source(
        documents.iter().map(|member| {
            (
                member.source_path.as_path(),
                member.materialized_path.as_path(),
                member.source.as_str(),
                member.code.as_str(),
                member.mapping_kind == MaterializedSourceMappingKind::Synthetic,
            )
        }),
        &target,
        input.component_text,
    )?;
    let file = name(input.generated_file)?;
    let first_host = "__vize_slot_host_9_instances";
    let second_host = "__vize_slot_props_host_9_instances";
    Ok(UsageRequest {
        usage_id: "original27:constructor-public-props-discrepancy:WitnessChild".into(),
        authored_file: name(&input.root.join("src/Tiny.vue"))?,
        authored_source: input.authored_text.into(),
        authored_usage: anchor(
            input.authored_text,
            "<WitnessChild :value=\"1\">",
            "WitnessChild",
        )?,
        config_file: name(input.config)?,
        generated: SourceExpectation {
            file: file.clone(),
            path: file,
            text: input.generated_text.into(),
        },
        component_reference: anchor(input.generated_text, "import WitnessChild", "WitnessChild")?,
        vue_module: anchor(&component.text, "'vue'", "vue")?,
        hosts: [
            anchor(
                input.generated_text,
                cstr!("const {first_host} =").as_str(),
                first_host,
            )?,
            anchor(
                input.generated_text,
                cstr!("const {second_host} =").as_str(),
                second_host,
            )?,
        ],
        component_source: component,
    })
}

fn hash(bytes: &[u8]) -> String {
    let mut encoded = String::with_capacity(64);
    for byte in Sha256::digest(bytes) {
        encoded.push_str(cstr!("{byte:02x}").as_str());
    }
    encoded
}

fn persist(file: &mut fs::File, value: &Value) -> Result<(), Failure> {
    let bytes = serde_json::to_vec_pretty(value).map_err(Failure::Json)?;
    if let Err(error) = file.write_all(&bytes).and_then(|_| file.sync_all()) {
        // Before the parent assertion, also preserve the complete SDK receipt
        // on the already-captured test stderr if the owned file cannot finish.
        let _ = std::io::stderr().write_all(&bytes);
        return Err(Failure::Io(error));
    }
    Ok(())
}

pub(super) fn capture_if_requested(
    session: &ProjectSession,
    old_snapshot: &ManagedSnapshot,
    evidence: &Path,
    input: Input<'_>,
) -> Result<(), Failure> {
    // No-env returns before looking at the owner/roster or issuing any SDK call.
    let Some(output) = output_path(std::env::var_os(OUTPUT), evidence)? else {
        return Ok(());
    };
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&output)
        .map_err(Failure::Io)?;
    let caller = json!({"source":std::env::var("NATIVE_ORIGIN_SOURCE").ok(),
        "tree":std::env::var("NATIVE_ORIGIN_TREE").ok(),"oldSnapshot":old_snapshot.handle,
        "selectedProject":session.project(),"authoredSource":input.authored_text,
        "generatedFile":input.generated_file.to_str(),"generatedText":input.generated_text,
        "componentAuthoredSource":input.component_text,"configName":input.config.to_str(),
        "consumerSourceSha256":hash(include_bytes!("constructor_vue_return_consumer.rs")),
        "producerSourceSha256":hash(include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/corsa_bridge/constructor_vue_proof.rs"))),
        "producerQueriesSha256":hash(include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/corsa_bridge/constructor_vue_proof/query.rs"))),
        "mappingScope":"caller tag-name request anchor only; no native authored-to-generated or call-node mapping admission"});
    let prepared = if !std::ptr::eq(session.snapshot_lease().as_ref(), old_snapshot) {
        Err(Failure::Input(
            "sidecar must borrow the original snapshot owner",
        ))
    } else {
        request(&input)
    };
    let request = match prepared {
        Ok(request) => request,
        Err(error) => {
            persist(
                &mut file,
                &json!({"schema":"vize.constructor-vue-sidecar.v1","caller":caller,
                "preflightError":cstr!("{error}"),"nativeAdmission":false,"extraNativeCalls":0}),
            )?;
            return Err(error);
        }
    };
    let caller_request = request.clone();
    match block_on(capture(session, request)) {
        Ok(observation) => persist(
            &mut file,
            &json!({"schema":"vize.constructor-vue-sidecar.v1",
            "caller":caller,"callerRequest":caller_request,"observation":observation,
            "resultKind":"Ok; see all retained unavailable fields","nativeAdmission":false,
            "diagnosticRepair":false,"responseScope":"complete typed SDK observations; not whole wire"}),
        ),
        Err(failure) => {
            let stored = persist(
                &mut file,
                &json!({"schema":"vize.constructor-vue-sidecar.v1",
                "caller":caller,"callerRequest":caller_request,"partial":failure.partial,
                "nativeErrorDebug":cstr!("{:?}",failure.error),"nativeErrorDisplay":cstr!("{}",failure.error),
                "errorRepresentation":"presentation only; original CorsaError returned in Failure::Native",
                "nativeAdmission":false,"diagnosticRepair":false,
                "responseScope":"complete typed partial observation; not whole wire"}),
            );
            Err(Failure::Native(failure.error, stored.err().map(Box::new)))
        }
    }
}

#[test]
fn optional_output_is_absent_or_exact_owned_child_only() {
    let evidence = Path::new("/owned/evidence");
    assert_eq!(output_path(None, evidence).unwrap(), None);
    assert_eq!(
        output_path(Some(evidence.join(FILE).into_os_string()), evidence).unwrap(),
        Some(evidence.join(FILE))
    );
    for raw in [
        "/other/constructor-vue-return.json",
        "/owned/evidence/other.json",
        "relative.json",
    ] {
        assert_eq!(
            cstr!("{}", output_path(Some(raw.into()), evidence).unwrap_err()),
            "sidecar output must be the exact existing evidence child"
        );
    }
}

#[test]
fn caller_anchors_keep_unicode_crlf_offsets_and_refuse_ambiguity() {
    let source = "😀\r\nconst host = 1;";
    let value = anchor(source, "const host =", "host").unwrap();
    assert_eq!((value.start, value.end, value.position), (10, 14, 10));
    assert_eq!(value.text.as_str(), "host");
    for source in ["const other = 1;", "const host = 1; const host = 2;"] {
        assert_eq!(
            cstr!("{}", anchor(source, "const host =", "host").unwrap_err()),
            "request anchor context is absent or ambiguous"
        );
    }
    assert_eq!(
        cstr!("{}", anchor("host host", "host host", "host").unwrap_err()),
        "request anchor token is absent or ambiguous"
    );
}

#[test]
fn component_roster_uses_exact_owner_not_basename_and_preserves_native_code() {
    let original = Path::new("/p/component.ts");
    let materialized = Path::new("/owned/component.ts");
    let member = (
        original,
        materialized,
        "authored\r\n",
        "generated\r\n",
        false,
    );
    let actual = component_source([member], original, "authored\r\n").unwrap();
    assert_eq!(actual.file.as_str(), "/owned/component.ts");
    assert_eq!(actual.path, actual.file);
    assert_eq!(actual.text.as_str(), "generated\r\n");
    for members in [
        vec![],
        vec![member, member],
        vec![(
            Path::new("/foreign/component.ts"),
            materialized,
            "authored\r\n",
            "generated\r\n",
            false,
        )],
        vec![(
            original,
            materialized,
            "authored\r\n",
            "generated\r\n",
            true,
        )],
        vec![(original, materialized, "authored\n", "generated\r\n", false)],
    ] {
        assert_eq!(
            cstr!(
                "{}",
                component_source(members, original, "authored\r\n").unwrap_err()
            ),
            "component must have one exact owned materialized source"
        );
    }
}

#[test]
fn sidecar_persistence_keeps_complete_values_and_refuses_overwrite() {
    let directory = tempfile::tempdir().unwrap();
    let output = directory.path().join(FILE);
    let original =
        json!({"nativeAdmission":false,"text":"雪😀\r\n","unknown":null,"rangeUnavailable":true});
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&output)
        .unwrap();
    persist(&mut file, &original).unwrap();
    assert_eq!(
        serde_json::from_slice::<Value>(&fs::read(&output).unwrap()).unwrap(),
        original
    );
    let error = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&output)
        .unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::AlreadyExists);
    assert_eq!(
        serde_json::from_slice::<Value>(&fs::read(&output).unwrap()).unwrap(),
        original
    );
}
