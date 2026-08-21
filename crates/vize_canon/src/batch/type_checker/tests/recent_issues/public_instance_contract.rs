//! The call-site prop surface and the normalized public instance are distinct
//! contracts (#4034). JSX needs camel/kebab aliases, while `InstanceType` and
//! declaration consumers must retain canonical required props and strict
//! emits. A single generic constructor chooses between those contracts using
//! an `unknown` default that cannot be confused with authored input.

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
const genericRef = useTemplateRef<InstanceType<typeof Generic>>('generic')

publicRef.value?.$emit('select', 'ok')
callableRef.value?.$emit('commit', 'ok')
runtimeObjectRef.value?.$emit('save', 'ok')
genericRef.value?.$emit('pick', 'ok')
publicRef.value?.close(true)

// @ts-expect-error template-ref record emit payload stays exact
publicRef.value?.$emit('select', 1)
// @ts-expect-error template-ref callable payload stays exact
callableRef.value?.$emit('commit', 1)
// @ts-expect-error template-ref runtime payload stays exact
runtimeObjectRef.value?.$emit('save', 1)
// @ts-expect-error template-ref generic payload stays exact
genericRef.value?.$emit('pick', 1)
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

    let mut checker = BatchTypeChecker::new(&project_root).unwrap();
    checker.scan_project().unwrap();
    let out_dir = project_root.join("types");
    let emitted = checker
        .emit_declarations(&DeclarationEmitOptions::new(out_dir.clone()))
        .expect("public-instance declarations should emit");
    let public_dts = std::fs::read_to_string(out_dir.join("Public.vue.d.ts")).unwrap();
    assert!(
        public_dts.contains(
            "type __VizeComponentConstructor = new <__VizeAuthoredProps = unknown>(props?: __VizeAuthoredProps & __VizeComponentInput<Props, __EmitProps<Emits>, __VizeAuthoredProps>, ...args: any[])"
        ) && public_dts.contains(
            "declare const __vize_component__: {\n    __vizeEmitProps?: __VizeStaticEmitProps;\n    readonly __vizeRawProps?: Props;\n    readonly __vizeSlots?: Partial<Slots>;\n} & __VizeComponentConstructor & __VizeVueComponentOptions;"
        )
            && !public_dts.contains("__VizeComponentInputConstructor")
            && !public_dts.contains("__VizeComponentPublicConstructor"),
        "one constructor must own authored, public-instance, and incremental raw-props contracts:\n{public_dts}"
    );
    assert!(
        public_dts.contains("$props: Props & __EmitProps<Emits>;")
            && public_dts.contains("$emit: __VizeStrictPublicEmit<Emits>;")
            && public_dts.contains("} & __VizeComponentPublicBase &"),
        "emitted public instance must own props/emits and adapt the Vue base:\n{public_dts}"
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
