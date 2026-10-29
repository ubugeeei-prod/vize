//! The call-site prop surface and the normalized public instance are distinct
//! contracts (#4034). JSX needs camel/kebab aliases, while `InstanceType` and
//! declaration consumers must retain canonical required props and strict
//! emits. A single generic constructor chooses between those contracts using
//! an `unknown` default that cannot be confused with authored input.
#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]

use super::super::{
    BatchTypeChecker, DeclarationEmitOptions, create_project_case, relative_path,
    resolve_test_tsgo_binary, snapshot_project_diagnostics, with_workspace_node_modules_override,
};
use std::path::{Path, PathBuf};
use std::process::Command;

const PUBLIC: &str = r#"<script setup lang="ts">
defineProps<{
  someValue: string
  optionalValue?: boolean
}>()

defineModel<number>({ required: true })
defineModel<string>('title', { default: 'ready' })
defineModel<boolean>('enabled')

defineEmits<{
  select: [value: string]
  clear: []
}>()

defineSlots<{
  default(props: { value: string }): unknown
}>()

function close(force = false): boolean {
  return force
}

defineExpose({ close })
</script>

<template>
  <button @click="$emit('select', someValue)">
    <slot :value="someValue" />
  </button>
</template>
"#;

const CALLABLE: &str = r#"<script setup lang="ts">
interface CallableEmits {
  (event: 'commit', value: string): void
  (event: 'cancel'): void
}

defineEmits<CallableEmits>()
</script>
"#;

const RUNTIME_OBJECT: &str = r#"<script setup lang="ts">
defineEmits({
  save: (value: string) => value.length > 0,
  reset: () => true,
})
</script>
"#;

const RUNTIME_ARRAY: &str = r#"<script setup lang="ts">
defineEmits(['open', 'close'] as const)
</script>
"#;

const GENERIC: &str = r#"<script setup lang="ts" generic="T extends string = string">
const props = defineProps<{ item: T }>()
defineEmits<{ pick: [value: T] }>()
defineSlots<{ default(props: { item: T }): unknown }>()
defineExpose({ current: (): T => props.item })
</script>

<template>
  <slot :item="item" />
</template>
"#;

const UNION: &str = r#"<script setup lang="ts">
defineProps<
  | { kind: 'text'; textValue: string }
  | { kind: 'count'; countValue: number }
>()
</script>
"#;

const NATIVE_LISTENER_PROP: &str = r#"<script setup lang="ts">
defineProps<{ onClick: (value: string) => void }>()
</script>
"#;

const NO_EMITS: &str = r#"<script setup lang="ts">
function ping(): string {
  return 'pong'
}

defineExpose({ ping })
</script>
"#;

const AUGMENTATION: &str = r#"import 'vue'
declare module 'vue' {
  interface ComponentCustomProperties {
    $service: { ping(value: string): number }
  }
}
"#;

const SOURCE_CONSUMER: &str = include_str!("public_instance_source.tsx");

const TEMPLATE_REF_CONSUMER: &str = r#"<script setup lang="ts">
import { useTemplateRef } from 'vue'
import Public from './Public.vue'
import Callable from './Callable.vue'
import RuntimeObject from './RuntimeObject.vue'
import Generic from './Generic.vue'

const publicRef = useTemplateRef<InstanceType<typeof Public>>('public')
const callableRef = useTemplateRef<InstanceType<typeof Callable>>('callable')
const runtimeObjectRef = useTemplateRef<InstanceType<typeof RuntimeObject>>('runtime-object')
type GenericExposed = Parameters<NonNullable<ReturnType<typeof Generic>['__ctx']>['expose']>[0]
const genericRef = useTemplateRef<GenericExposed>('generic')

publicRef.value?.$emit('select', 'ok')
callableRef.value?.$emit('commit', 'ok')
runtimeObjectRef.value?.$emit('save', 'ok')
const current: string | undefined = genericRef.value?.current()
void current
publicRef.value?.close(true)

// @ts-expect-error template-ref record emit payload stays exact
publicRef.value?.$emit('select', 1)
// @ts-expect-error template-ref callable payload stays exact
callableRef.value?.$emit('commit', 1)
// @ts-expect-error template-ref runtime payload stays exact
runtimeObjectRef.value?.$emit('save', 1)
// @ts-expect-error generic exposes do not acquire a public-instance $emit
genericRef.value?.$emit('pick', 'ok')
// @ts-expect-error template-ref exposed member stays exact
publicRef.value?.close('yes')
</script>

<template>
  <Public ref="public" :model-value="1" some-value="ok">ok</Public>
  <Callable ref="callable" />
  <RuntimeObject ref="runtime-object" />
  <Generic ref="generic" item="ok">ok</Generic>
</template>
"#;

const DOWNSTREAM: &str = include_str!("public_instance_downstream.tsx");

const NESTED_COMPONENT: &str = r#"<script setup lang="ts">
defineProps<{
  label: string
}>()
defineEmits<{
  save: [value: string]
}>()
</script>
"#;

const WIDEN_TRIGGER: &str = r#"import { runtimeValue } from '../shared/runtime'
export const buildFlag: boolean = runtimeValue
"#;

const OUTSIDE_RUNTIME: &str = r#"export const runtimeValue = true
"#;

fn write_tsconfig(project_root: &Path) {
    std::fs::write(
        project_root.join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "jsx": "preserve",
    "jsxImportSource": "vue",
    "rootDir": "src",
    "noEmit": true
  },
  "include": ["src/**/*"]
}"#,
    )
    .unwrap();
}

fn compiler_output(command: &mut Command) -> (bool, String) {
    let output = command.output().expect("type compiler should run");
    let mut rendered = String::from_utf8_lossy(&output.stdout).into_owned();
    rendered.push_str(&String::from_utf8_lossy(&output.stderr));
    (output.status.success(), rendered)
}

pub(super) fn resolve_javascript_tsc() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("VIZE_TEST_JAVASCRIPT_TSC").map(PathBuf::from)
        && path.is_file()
    {
        return Some(path);
    }
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)?;
    let candidate = workspace_root
        .join("tests")
        .join("node_modules")
        .join(".bin")
        .join("tsc");
    candidate.is_file().then_some(candidate)
}

pub(super) fn assert_downstream_compiler(compiler: &Path, downstream: &Path) {
    let (success, output) = compiler_output(
        Command::new(compiler)
            .arg("--ignoreConfig")
            .arg("--noEmit")
            .arg("--strict")
            .arg("--jsx")
            .arg("preserve")
            .arg("--jsxImportSource")
            .arg("vue")
            .arg("--module")
            .arg("preserve")
            .arg("--moduleResolution")
            .arg("bundler")
            .arg("--target")
            .arg("ES2022")
            .arg("--lib")
            .arg("ES2022,DOM")
            .arg("--skipLibCheck")
            .arg("--pretty")
            .arg("false")
            .arg(downstream),
    );
    assert!(
        success,
        "{} rejected emitted declarations:\n{output}",
        compiler.display()
    );
    assert!(
        !output.contains("Unused '@ts-expect-error'") && !output.contains("TS2578"),
        "{} left a negative oracle unused:\n{output}",
        compiler.display()
    );
}

#[test]
fn public_instance_contract_survives_source_and_declaration_consumers() {
    let Some(tsgo) = resolve_test_tsgo_binary() else {
        return;
    };
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root should exist");
    let test_node_modules = workspace_root.join("tests").join("node_modules");
    assert!(
        test_node_modules
            .join("vue/jsx-runtime/index.d.ts")
            .is_file(),
        "the strict JSX oracle requires the installed real Vue runtime"
    );
    let project_root = with_workspace_node_modules_override(
        Some(
            test_node_modules
                .to_str()
                .expect("test node_modules path should be UTF-8"),
        ),
        || {
            create_project_case(
                "public-instance-contract",
                &[
                    ("src/Public.vue", PUBLIC),
                    ("src/Callable.vue", CALLABLE),
                    ("src/RuntimeObject.vue", RUNTIME_OBJECT),
                    ("src/RuntimeArray.vue", RUNTIME_ARRAY),
                    ("src/Generic.vue", GENERIC),
                    ("src/Union.vue", UNION),
                    ("src/NativeListenerProp.vue", NATIVE_LISTENER_PROP),
                    ("src/NoEmits.vue", NO_EMITS),
                    ("src/nested/Nested.vue", NESTED_COMPONENT),
                    ("src/vue.d.ts", AUGMENTATION),
                    ("src/Consumer.tsx", SOURCE_CONSUMER),
                    ("src/Parent.vue", TEMPLATE_REF_CONSUMER),
                ],
            )
        },
    );
    write_tsconfig(&project_root);

    let diagnostics = snapshot_project_diagnostics(&project_root)
        .expect("source public-instance project should typecheck");
    assert_eq!(
        diagnostics,
        Vec::new(),
        "source contract drifted: {diagnostics:#?}"
    );

    std::fs::create_dir_all(project_root.join("shared")).unwrap();
    std::fs::write(project_root.join("src/widen-trigger.ts"), WIDEN_TRIGGER).unwrap();
    std::fs::write(project_root.join("shared/runtime.ts"), OUTSIDE_RUNTIME).unwrap();

    let mut checker = BatchTypeChecker::new(&project_root).unwrap();
    let mut declaration_roots = walkdir::WalkDir::new(project_root.join("src"))
        .into_iter()
        .map(Result::unwrap)
        .filter(|entry| entry.file_type().is_file())
        .map(walkdir::DirEntry::into_path)
        .collect::<Vec<_>>();
    declaration_roots.sort();
    checker.scan_paths(&declaration_roots).unwrap();
    let out_dir = project_root.join("types");
    let emitted = checker
        .emit_declarations(&DeclarationEmitOptions::new(out_dir.clone()).with_declaration_map(true))
        .expect("public-instance declarations should emit");
    let public_dts = std::fs::read_to_string(out_dir.join("Public.vue.d.ts")).unwrap();
    assert!(
        public_dts.contains(
            "type $ComponentConstructor = new <$AuthoredProps = unknown>(props?: $AuthoredProps & $ComponentInput<Props, import(\"./__vize_helpers.js\").__EmitProps<Emits>, $AuthoredProps>, ...args: any[])"
        ) && public_dts.contains(
            "declare const __vize_component__: {\n    __vizeEmitProps?: $StaticEmitProps;\n    readonly __vizeRawProps?: Props;\n    readonly __vizeSlots?: Partial<Slots>;\n} & $ComponentConstructor & $VueComponentOptions;"
        )
            && !public_dts.contains("$ComponentInputConstructor")
            && !public_dts.contains("$ComponentPublicConstructor"),
        "one constructor must own authored, public-instance, and incremental raw-props contracts:\n{public_dts}"
    );
    assert!(
        public_dts.contains("$props: Props & import(\"./__vize_helpers.js\").__EmitProps<Emits>;")
            && public_dts.contains("$emit: $PublicEmit<Emits>;")
            && public_dts.contains("$slots: $PublicSlots<$Slots>;")
            && public_dts.contains("} & $ComponentPublicBase &"),
        "emitted public instance must own props/emits and adapt the Vue base:\n{public_dts}"
    );
    assert_eq!(
        public_dts.matches("../__vize_helpers.js").count(),
        0,
        "a rootDir relocation must resolve helpers from the final root output:\n{public_dts}"
    );
    assert_eq!(
        public_dts.matches("vize:virtual-ts").count(),
        0,
        "published declarations cannot retain the private checker module:\n{public_dts}"
    );

    let nested_dts_path = out_dir.join("nested/Nested.vue.d.ts");
    let nested_dts = std::fs::read_to_string(&nested_dts_path).unwrap();
    assert_eq!(
        nested_dts
            .matches("import(\"../__vize_helpers.js\").__EmitProps")
            .count(),
        5,
        "a nested retained SFC must resolve the helper from its final published depth:\n{nested_dts}"
    );
    assert_eq!(nested_dts.matches("vize:virtual-ts").count(), 0);
    assert!(
        !out_dir.join("shared/runtime.d.ts").exists(),
        "an inferred dependency outside rootDir must not become public output"
    );
    assert_eq!(
        std::fs::read_to_string(out_dir.join("widen-trigger.d.ts")).unwrap(),
        "export declare const buildFlag: boolean;\n//# sourceMappingURL=widen-trigger.d.ts.map",
        "the widening trigger must not retain a path to the pruned dependency"
    );

    let nested_map_path = out_dir.join("nested/Nested.vue.d.ts.map");
    let nested_map: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&nested_map_path).unwrap()).unwrap();
    assert_eq!(nested_map["file"], "Nested.vue.d.ts");
    assert_eq!(nested_map["sourceRoot"], "");
    let nested_sources = nested_map["sources"].as_array().unwrap();
    assert_eq!(nested_sources.len(), 1);
    let nested_source = nested_sources[0].as_str().unwrap();
    assert_eq!(
        std::fs::canonicalize(nested_map_path.parent().unwrap().join(nested_source)).unwrap(),
        std::fs::canonicalize(project_root.join("src/nested/Nested.vue")).unwrap(),
        "the relocated declaration map must still resolve to the authored SFC"
    );

    let downstream_dir = project_root.join("downstream");
    std::fs::create_dir_all(&downstream_dir).unwrap();
    let downstream = downstream_dir.join("verify.tsx");
    std::fs::write(&downstream, DOWNSTREAM).unwrap();
    assert_downstream_compiler(&tsgo, &downstream);
    if let Some(tsc) = resolve_javascript_tsc() {
        assert_downstream_compiler(&tsc, &downstream);
    }

    let emitted_paths: Vec<_> = emitted
        .files
        .iter()
        .map(|file| relative_path(&out_dir, &file.path))
        .collect();
    assert!(
        emitted_paths
            .iter()
            .any(|path| path == "__vize_helpers.d.ts"),
        "declaration emit must use the shared helper semantic source: {emitted_paths:?}"
    );
    let _ = std::fs::remove_dir_all(&project_root);
}
