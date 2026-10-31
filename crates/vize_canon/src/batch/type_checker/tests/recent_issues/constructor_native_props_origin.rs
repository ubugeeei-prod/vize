//! Opt-in metadata probe; neither declaration equality nor this test grants props authority.
#![expect(
    clippy::disallowed_types,
    reason = "the SDK returns owned standard strings"
)]

use crate::batch::BatchTypeChecker;
use corsa::{
    api::{ApiMode, ApiSpawnConfig, ProjectSession},
    runtime::block_on,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{fs, path::Path};

mod query;

// Exact private case from frozen a8 original27/58; also retained by the 3bd capture.
const COMPONENT: &str = r#"import { defineComponent, type SlotsType, type VNode, type PropType } from 'vue';
const VueChild = defineComponent({
  props: { value: { type: Number, required: true }, render: Function as PropType<(value: number) => string> },
  slots: Object as SlotsType<{ default(props: { item: number }): VNode[] }>,
});
type Instance = InstanceType<typeof VueChild>;
type Typed<T> = Omit<Instance, '$props' | '$slots'> & {
  readonly $props: Readonly<{ value: T }>;
  readonly $slots: { default(props: { item: T }): VNode[] };
};
export declare const WitnessChild: { new(props: { value: number; internal: string }): Instance };
export default WitnessChild;
"#;
const TINY: &str = r#"<script setup lang="ts">
import WitnessChild from '../component';
type IsAny<T> = 0 extends 1 & T ? true : false;
type IsNever<T> = [T] extends [never] ? true : false;
type IsUnknown<T> = IsAny<T> extends true ? false : unknown extends T ? keyof T extends never ? true : false : false;
type Exact<A,B> = (<T>()=>T extends A?1:2) extends (<T>()=>T extends B?1:2) ? true : false;
type SlotReport<T> = IsAny<T> extends true ? 'any' : IsNever<T> extends true ? 'never' : IsUnknown<T> extends true ? 'unknown' : Exact<T, number> extends true ? 'number' : 'other';
function slotReport<T>(_value: T): SlotReport<T> { throw new Error('witness'); }
function expectNumber(_value: 'number'): void {}
</script>
<template>
  <WitnessChild :value="1">
    <template #default="{ item }">{{ expectNumber(slotReport(item)) }}</template>
  </WitnessChild>
</template>
"#;
const CONFIG: &str = r#"{
  "compilerOptions": {
    "strict": true,
    "noEmit": true,
    "skipLibCheck": true,
    "module": "ESNext",
    "moduleResolution": "Bundler",
    "target": "ES2024",
    "lib": [
      "ES2024",
      "DOM"
    ],
    "types": [],
    "incremental": false
  },
  "vueCompilerOptions": {
    "strictTemplates": true,
    "checkUnknownProps": true,
    "checkUnknownComponents": false
  },
  "include": [
    "**/*.vue",
    "**/component.ts"
  ]
}
"#;

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn image(path: &Path) -> Value {
    let bytes = fs::read(path).expect("probe-owned file remains readable");
    json!({"path":path,"bytes":bytes.len(),"sha256":sha256(&bytes)})
}

#[test]
fn frozen_native_origin_inputs_remain_byte_exact() {
    assert_eq!(
        sha256(COMPONENT.as_bytes()),
        "a9cc56b2a8b615bc61b970797072119722d443a54abc4b8602271d2b6be12d66"
    );
    assert_eq!(
        sha256(TINY.as_bytes()),
        "3f8deafae6a3564ac4b98c694f72c558e2af74a922a0ff1582fcfe9a2a5ac455"
    );
    assert_eq!(
        sha256(CONFIG.as_bytes()),
        "a619867d3932c065dd60789380f6014b8c6572d93fdcb7f4a2a57ed6a3d8c965"
    );
}

#[cfg(unix)]
#[test]
fn current_native_instance_props_declaration_origins() {
    let Some(output) = std::env::var_os("VIZE_NATIVE_PROPS_ORIGIN_OUTPUT") else {
        return;
    };
    assert!(std::env::var_os("VIZE_TEST_DISABLE_TSGO").is_none());
    assert_eq!(std::env::var("VIZE_TEST_REQUIRE_TSGO").as_deref(), Ok("1"));
    let native = fs::canonicalize(std::env::var_os("CORSA_PATH").expect("explicit native image"))
        .expect("native image exists");
    let modules = fs::canonicalize(
        std::env::var_os("VIZE_NATIVE_PROPS_ORIGIN_MODULES")
            .expect("explicit frozen provider modules"),
    )
    .expect("provider exists");
    let vue: Value =
        serde_json::from_slice(&fs::read(modules.join("vue/package.json")).unwrap()).unwrap();
    assert_eq!(vue["version"], "3.5.42");
    frozen_native_origin_inputs_remain_byte_exact();
    let output = Path::new(&output);
    assert!(!output.exists(), "never overwrite a native probe receipt");
    let evidence = output.parent().expect("receipt parent");
    fs::create_dir_all(evidence).unwrap();
    let temporary = tempfile::tempdir().unwrap();
    let root = temporary.path();
    fs::create_dir(root.join("src")).unwrap();
    fs::write(root.join("component.ts"), COMPONENT).unwrap();
    fs::write(root.join("src/Tiny.vue"), TINY).unwrap();
    fs::write(root.join("tsconfig.json"), CONFIG).unwrap();
    std::os::unix::fs::symlink(&modules, root.join("node_modules")).unwrap();
    let mut checker =
        BatchTypeChecker::with_options_and_corsa_path(root, Default::default(), Some(&native))
            .expect("construct without running diagnostics");
    checker
        .scan_project()
        .expect("generate unchanged current VTS");
    checker
        .project
        .materialize()
        .expect("materialize unchanged current VTS");
    let files = checker.virtual_files();
    let generated = files
        .iter()
        .filter(|file| file.original_path == root.join("src/Tiny.vue"))
        .collect::<Vec<_>>();
    assert_eq!(generated.len(), 1);
    let generated = generated[0];
    fs::write(evidence.join("generated.ts"), generated.content.as_bytes()).unwrap();
    fs::write(evidence.join("component.ts"), COMPONENT).unwrap();
    fs::write(evidence.join("Tiny.vue"), TINY).unwrap();
    fs::write(evidence.join("tsconfig.json"), CONFIG).unwrap();
    let config = checker.project.generated_tsconfig_path();
    fs::copy(&config, evidence.join("materialized-tsconfig.json")).unwrap();
    let config_name = config.to_str().expect("UTF8 config");
    let spawned = block_on(ProjectSession::spawn(
        ApiSpawnConfig::new(native.to_str().unwrap())
            .with_mode(ApiMode::AsyncJsonRpcStdio)
            .with_cwd(checker.project.virtual_root()),
        config_name,
        None,
    ));
    let mut calls = Vec::new();
    let (result, cleanup) = match spawned {
        Ok(session) => {
            let client = session.client().clone();
            let lease = session.snapshot_lease();
            let project = session.project().clone();
            calls.push(
                json!({"label":"explicitProjectConfig","expectedConfig":config_name,
                "actualProject":project}),
            );
            let mut result = if project.config_file_name != config_name {
                Err("native project does not own the requested generated config".into())
            } else {
                query::observe(
                    &client,
                    &lease,
                    &project,
                    &generated.virtual_path,
                    generated.content.as_str(),
                    COMPONENT,
                    &mut calls,
                )
            };
            let current = std::sync::Arc::ptr_eq(&lease, &session.snapshot_lease())
                && session.project() == &project;
            calls.push(json!({"label":"sameOwnerAfterQueries","current":current,
                "snapshot":lease.handle,"project":project}));
            if !current {
                result = Err("project snapshot changed during the probe".into());
            }
            // Do not explicitly release a lease still owned by ProjectSession.
            drop(lease);
            drop(session);
            let closed = block_on(client.close());
            let cleanup = json!({"snapshotLeaseDropped":true,"sessionDropped":true,
                "clientClose":closed.as_ref().map(|_| ()).map_err(|error|error.to_string()),
                "quiescenceClaim":false});
            (result, cleanup)
        }
        Err(error) => (
            Err(format!("project session unavailable: {error}")),
            json!({"sessionCreated":false,"quiescenceClaim":false}),
        ),
    };
    let receipt = json!({"schemaVersion":1,"scope":"one frozen imported original27 case; native metadata observation only",
        "fixtureSource":"a8ce1ec166a53b380e74dbf7189cc8578c80c676",
        "historicalCurrentSource":"3bd645ac235c2efea1e23b2ccc104aaa18f720e6",
        "source":std::env::var("NATIVE_ORIGIN_SOURCE").ok(),
        "tree":std::env::var("NATIVE_ORIGIN_TREE").ok(),"native":image(&native),
        "vuePackage":image(&modules.join("vue/package.json")),
        "generated":image(&evidence.join("generated.ts")),"inputRoot":root,
        "materializedConfig":image(&config),"requests":calls,"result":result,
        "cleanup":cleanup,"responseScope":"complete typed SDK DTOs; not whole transport frames",
        "flagInterpretation":"UNPROVED; numeric flags/checkFlags are retained opaque",
        "diagnosticsRequested":false,"nativeAdmission":false,"productionChanged":false});
    fs::write(output, serde_json::to_vec_pretty(&receipt).unwrap()).unwrap();
    assert!(
        receipt["result"].get("Ok").is_some(),
        "origin metadata probe refused: {receipt}"
    );
    assert!(
        receipt["cleanup"]["clientClose"].get("Ok").is_some(),
        "native close failed: {receipt}"
    );
}
