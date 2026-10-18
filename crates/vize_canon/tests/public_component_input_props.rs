//! #9498: listener inference must retain Vue's augmented public prop types.
#![cfg(feature = "native")]

use std::{path::Path, process::Command};

use serde_json::{Value, json};
use vize_canon::batch::generate_vue_content_mapper_transform;
use vize_l0::cstr;

const FIXTURE: &str =
    include_str!("../../../tests/_fixtures/content-mapper/escaton-public-props-9498.json");
const CONSUMER: &str = r#"import Child from './Child.vue'
import Required from './Required.vue'
import Union from './Union.vue'

new Child({ hidden: true, onClose: () => {} })
new Child({ globalToken: 'session', onClose: () => {} })
new Child({ key: 'child', ref: 'child', onClose: () => {} })
// @ts-expect-error the augmented boolean remains typed during listener inference
new Child({ hidden: 'wrong', onClose: () => {} })
// @ts-expect-error ComponentCustomProps also retains its value type
new Child({ globalToken: 42, onClose: () => {} })
// @ts-expect-error public props must not admit arbitrary component inputs
new Child({ definitelyNotAProp: true, onClose: () => {} })
new Required({ amount: 1, hidden: true, onClose: () => {} })
// @ts-expect-error public props do not make declared required props optional
new Required({ hidden: true, onClose: () => {} })
// @ts-expect-error declared props keep their value type
new Required({ amount: 'wrong', hidden: true, onClose: () => {} })
// @ts-expect-error declared listener payloads remain typed
new Required({ amount: 1, onClose: (value: string) => {} })
new Union({ kind: 'count', count: 1, hidden: true, onClose: () => {} })
new Union({ kind: 'text', text: 'ok', hidden: true, onClose: () => {} })
// @ts-expect-error a public prop does not merge discriminated union arms
new Union({ kind: 'count', text: 'wrong', hidden: true, onClose: () => {} })
const instance = null as unknown as InstanceType<typeof Required>
const amount: number = instance.$props.amount
// @ts-expect-error the public instance still exposes the declared prop type
const wrong: string = instance.$props.amount
"#;
const HOST: &str = r#"
const fs = require('node:fs');
const path = require('node:path');
const { createRequire } = require('node:module');
const workspace = process.argv[1], directory = process.argv[2];
const fromTests = createRequire(path.join(workspace, 'tests/package.json'));
const ts = fromTests('typescript');
const vuePackage = fromTests.resolve('vue/package.json');
const fromVue = createRequire(vuePackage);
const fromRuntimeDom = createRequire(fromVue.resolve('@vue/runtime-dom/package.json'));
const runtimeCorePackage = fromRuntimeDom.resolve('@vue/runtime-core/package.json');
// Bind the augmentation to this Vue's dependency, not a neighboring workspace
// runtime-core version or an unresolved ambient import hidden by skipLibCheck.
fs.mkdirSync(path.join(directory, 'node_modules/@vue'), { recursive: true });
fs.symlinkSync(path.dirname(vuePackage), path.join(directory, 'node_modules/vue'), 'junction');
fs.symlinkSync(path.dirname(runtimeCorePackage),
  path.join(directory, 'node_modules/@vue/runtime-core'), 'junction');
const options = { strict: true, skipLibCheck: true, noEmit: true, types: [],
  target: ts.ScriptTarget.ESNext, jsx: ts.JsxEmit.Preserve, module: ts.ModuleKind.ESNext,
  moduleResolution: ts.ModuleResolutionKind.Bundler };
const modules = JSON.parse(fs.readFileSync(path.join(directory, 'modules.json'), 'utf8'));
const host = ts.createCompilerHost(options);
// This checks complete native modules with the declared JS compiler, not the
// external mapper protocol. The official TS7 carrier checks that separately.
host.resolveModuleNames = (names, containing) => names.map(name => {
  const virtual = modules[path.resolve(path.dirname(containing), name)];
  // The authored specifier is .vue, not a TypeScript-extension import. Return
  // the registered virtual module instead of resolving a fabricated .tsx import.
  if (virtual) return { resolvedFileName: virtual, extension: path.extname(virtual),
    isExternalLibraryImport: false };
  return ts.resolveModuleName(name, containing, options, host).resolvedModule;
});
const files = fs.readdirSync(directory).filter(file => /\.tsx?$/.test(file));
const program = ts.createProgram(files.map(file => path.join(directory, file)), options, host);
const diagnostics = ts.getPreEmitDiagnostics(program).map(d => ({
  code: d.code, category: d.category, start: d.start, length: d.length,
  file: d.file && path.relative(directory, d.file.fileName),
  message: ts.flattenDiagnosticMessageText(d.messageText, '\n')
}));
process.stdout.write(JSON.stringify({ typescript: ts.version,
  vue: fromTests('vue/package.json').version,
  runtimeCore: JSON.parse(fs.readFileSync(runtimeCorePackage, 'utf8')).version, diagnostics }));
"#;

#[test]
fn complete_native_component_modules_preserve_public_and_declared_input_contracts() {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let profile = std::env::var("NEXTEST_PROFILE").unwrap_or_else(|_| "pr".into());
    let evidence = workspace
        .join("target/nextest")
        .join(profile)
        .join("public-component-input-props");
    std::fs::create_dir_all(&evidence).unwrap();
    let project = tempfile::tempdir().unwrap();

    let fixture: Value = serde_json::from_str(FIXTURE).unwrap();
    let mut sources = fixture["originals"][0]["files"]
        .as_object()
        .unwrap()
        .clone();
    sources.insert(
        "WithoutListener.vue".into(),
        fixture["controls"][0]["files"]["App.vue"].clone(),
    );
    sources.insert("Consumer.ts".into(), CONSUMER.into());
    sources.insert(
        "env.d.ts".into(),
        fixture["controls"]
            .as_array()
            .unwrap()
            .iter()
            .find(|case| case["name"] == "public-props-9498-custom-props-listener")
            .unwrap()["files"]["env.d.ts"]
            .clone(),
    );
    for (name, props) in [
        ("Required.vue", "{ amount: number }"),
        (
            "Union.vue",
            "{ kind: 'count'; count: number } | { kind: 'text'; text: string }",
        ),
    ] {
        sources.insert(
            name.into(),
            cstr!(
                "<script setup lang=\"ts\">\ndefineProps<{props}>()\ndefineEmits<{{ close: [value: number] }}>()\n</script>\n"
            )
            .as_str()
            .into(),
        );
    }
    let mut modules = serde_json::Map::new();
    for (name, source) in &sources {
        let source = source.as_str().unwrap();
        std::fs::write(evidence.join(name), source).unwrap();
        if name.ends_with(".vue") {
            let transform = generate_vue_content_mapper_transform(Path::new(name), source)
                .expect("complete native mapper transform");
            std::fs::write(
                evidence.join(cstr!("{name}.transform.json")),
                serde_json::to_vec_pretty(&transform).unwrap(),
            )
            .unwrap();
            assert_eq!(transform.diagnostics, vec![], "{name}: {transform:#?}");
            let generated = cstr!("{name}{}", transform.extension);
            modules.insert(
                project.path().join(name).to_string_lossy().into_owned(),
                project
                    .path()
                    .join(&generated)
                    .to_string_lossy()
                    .into_owned()
                    .into(),
            );
            std::fs::write(project.path().join(&generated), transform.text.as_str()).unwrap();
            std::fs::write(evidence.join(&generated), transform.text.as_str()).unwrap();
        } else {
            std::fs::write(project.path().join(name), source).unwrap();
        }
    }
    let modules = serde_json::to_vec_pretty(&modules).unwrap();
    std::fs::write(project.path().join("modules.json"), &modules).unwrap();
    std::fs::write(evidence.join("modules.json"), &modules).unwrap();
    let output = Command::new("node")
        .arg("-e")
        .arg(HOST)
        .arg(workspace)
        .arg(project.path())
        .output()
        .expect("run the declared TypeScript dependency; never skip the semantic assertions");
    std::fs::write(evidence.join("stdout.bin"), &output.stdout).unwrap();
    std::fs::write(evidence.join("stderr.bin"), &output.stderr).unwrap();
    std::fs::write(
        evidence.join("process.json"),
        serde_json::to_vec_pretty(&json!({
            "sourceSha": std::env::var("GITHUB_SHA").ok(), "host": HOST,
            "exitCode": output.status.code(), "success": output.status.success(),
        }))
        .unwrap(),
    )
    .unwrap();
    assert!(output.status.success(), "{output:?}");
    assert_eq!(output.stderr, b"", "{output:?}");
    let observed: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(observed["typescript"], "6.0.3");
    assert!(observed["vue"].as_str().is_some());
    assert_eq!(observed["runtimeCore"], observed["vue"]);
    assert_eq!(observed["diagnostics"], json!([]), "{observed:#}");
}
