use super::super::scope::PrefixScope;
use super::prefix_slot_defaults;
use crate::emit::options::{BindingKind, BindingTable};
use serde_json::{Value, json};

fn binding_kind(kind: &str) -> BindingKind {
    match kind {
        "literal-const" => BindingKind::LiteralConst,
        "setup-const" => BindingKind::SetupConst,
        "setup-ref" => BindingKind::SetupRef,
        "setup-maybe-ref" => BindingKind::SetupMaybeRef,
        "setup-let" => BindingKind::SetupLet,
        "setup-reactive-const" => BindingKind::SetupReactiveConst,
        "props" => BindingKind::Props,
        "props-aliased" => BindingKind::PropsAliased,
        "data" => BindingKind::Data,
        "options" => BindingKind::Options,
        _ => panic!("unrecognized authored binding kind: {kind}"),
    }
}

#[cfg(test)]
#[test]
fn inline_immutable_setup_defaults_keep_exact_authored_parameter_packets() {
    let controls: Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/_fixtures/differential/compiler/slot-default-setup-bindings-8142/controls.json"
    )))
    .expect("independently authored controls");
    let mut packets = std::vec::Vec::new();
    for case in controls["cases"].as_array().expect("cases") {
        let table = case["bindings"].as_object().map(|bindings| {
            BindingTable::new(
                bindings.iter().map(|(name, kind)| {
                    (name.as_str(), binding_kind(kind.as_str().expect("kind")))
                }),
                [],
                true,
            )
        });
        let inline = case["inline"].as_bool().expect("inline");
        let scope = PrefixScope::new(table.as_ref(), true, false, inline);
        let output = prefix_slot_defaults(case["source"].as_str().expect("whole source"), &scope);
        packets.push(json!({
            "control": case,
            "actualInline": scope.inline(),
            "actualBindings": case["bindings"],
            "actualIsScriptSetup": table.as_ref().map(BindingTable::is_script_setup),
            "actualWholeParameters": output.as_str(),
        }));
    }
    // Retain every complete context/input/output before judging any product law.
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../../target/differential/slot-default-setup-bindings-8142/native-parameter-packets.json",
    );
    std::fs::create_dir_all(path.parent().expect("output directory"))
        .expect("create raw directory");
    std::fs::write(
        &path,
        serde_json::to_vec_pretty(&json!({
            "schema": 1,
            "plane": "source-native-parameter-unit",
            "producerPid": std::process::id(),
            "githubSha": std::env::var("GITHUB_SHA").ok(),
            "controls": controls,
            "packets": packets,
        }))
        .expect("whole packet serialization"),
    )
    .expect("retain every raw observation");

    assert_eq!(packets.len(), 24, "frozen complete control count");
    for packet in packets {
        assert_eq!(
            packet["actualWholeParameters"], packet["control"]["expectedWholeParameters"],
            "complete parameters for {}",
            packet["control"]["id"]
        );
    }
}
