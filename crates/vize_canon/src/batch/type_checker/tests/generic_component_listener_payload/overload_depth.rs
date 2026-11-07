use super::super::{create_project_case, resolve_test_tsgo_binary, snapshot_project_diagnostics};

#[test]
fn generic_emit_overload_expansion_stays_bounded() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "generic-emit-overload-depth",
        &[
            (
                "src/Child.vue",
                r#"<script setup lang="ts" generic="T extends keyof Payloads = any">
interface Payloads {
  text: string
  count: number
}

defineProps<{ type: T }>()
defineEmits<{
  <K extends keyof Payloads>(event: "done", value: Payloads[K]): void
  (event: "closed"): void
}>()
</script>

<template><div /></template>
"#,
            ),
            (
                "src/usage.ts",
                r#"import Child from "./Child.vue";

type StaticEventProps<C> = C extends { __vizeEmitProps?: infer P }
  ? NonNullable<P>
  : never;
type ExpandedEventProps = {
  [K in keyof StaticEventProps<typeof Child>]: StaticEventProps<typeof Child>[K]
};

declare const handlers: ExpandedEventProps;
handlers.onClosed?.();
"#,
            ),
        ],
    );

    let Some(snapshot) = snapshot_project_diagnostics(&project_root) else {
        let _ = std::fs::remove_dir_all(&project_root);
        return;
    };

    assert!(
        snapshot.iter().all(|(_, code, _)| *code != Some(2589)),
        "generic emit prop expansion should not exceed the checker depth: {snapshot:#?}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

// Keep the former bounded algorithm as an independent <=16-event oracle.
const LEGACY_OVERLOADS: &str = r#"type $LegacyOverloadUnionRecursive<TOverload, TPartialOverload = unknown, TDepth extends unknown[] = []> = TDepth['length'] extends 32 ? never : TOverload extends (...args: infer TArgs) => infer TReturn ? TPartialOverload extends TOverload ? never : $LegacyOverloadUnionRecursive<TPartialOverload & TOverload, TPartialOverload & ((...args: TArgs) => TReturn) & $OverloadProps<TOverload>, [...TDepth, unknown]> | ((...args: TArgs) => TReturn) : never;
type $LegacyOverloadUnion<TOverload extends (...args: any[]) => any> = Exclude<$LegacyOverloadUnionRecursive<(() => never) & TOverload>, TOverload extends () => never ? never : () => never>;
type $LegacyOverloadParameters<T extends (...args: any[]) => any> = Parameters<$LegacyOverloadUnion<T>>;
type __LegacyEmitOptions<T, S = __EmitShape<T>> = { [K in keyof S as K extends string ? K : never]: (...args: __EmitArgs<S, K>) => any } & (__EmitShape<T> extends (...args: any[]) => any ? $ParametersToFns<$LegacyOverloadParameters<__EmitShape<T>>> : {});
type __LegacyEmitProps<T> = { [K in keyof __LegacyEmitOptions<T> & string as $HandlerKey<K> | `on${Capitalize<K>}`]?: __LegacyEmitOptions<T>[K] };
"#;

fn emit_overload_contract_source() -> std::string::String {
    let mut source = std::string::String::from("export {};\n");
    source.push_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/virtual_ts/helpers/declarations/core-types.d.ts.txt"
    )));
    source.push_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/virtual_ts/helpers/declarations/emit-overloads.d.ts.txt"
    )));
    source.push_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/virtual_ts/helpers/declarations/emit-props.d.ts.txt"
    )));
    source.push_str(LEGACY_OVERLOADS);
    source.push_str(
        r#"
type Equal<A, B> = (<T>() => T extends A ? 1 : 2) extends
  (<T>() => T extends B ? 1 : 2) ? true : false;
type Assert<T extends true> = T;
type GenericEvents<T> = __EmitFn<{ done: [value: T] }> &
  __EmitFn<{ 'update:value': [value: T] }>;
"#,
    );
    let shapes = [
        ("Empty", "{}"),
        ("Keyed", "{ done: [value: number]; closed: [] }"),
        (
            "Optional",
            "{ done: [value?: number]; rest: [head: string, ...tail: number[]] }",
        ),
        (
            "Kebab",
            "{ 'value-changed': [value: number]; 'update:modelValue': [value: string] }",
        ),
        ("Readonly", "{ readonly done: readonly [value: number] }"),
        (
            "CallableProps",
            "{ (event: 'done', value: number): void; (event: 'closed'): void; tag: 'owned' }",
        ),
        (
            "LiteralUnion",
            "(event: 'left' | 'right', value: number) => void",
        ),
        ("BroadString", "(event: string, ...args: any[]) => void"),
        ("Index", "Record<string, any[]>"),
        ("Any", "any"),
        ("Unknown", "unknown"),
        ("Never", "never"),
        (
            "GenericCall",
            "{ <T>(event: 'done', value: T): void; (event: 'closed'): void }",
        ),
        (
            "Union",
            "((event: 'left', value: number) => void) | ((event: 'right', value: string) => void)",
        ),
        (
            "Collision",
            "__EmitFn<{ 'update:m0': [value: number] }> & __EmitFn<{ 'update:m0': [value: string]; 'update:m1': [value: boolean] }>",
        ),
        ("GenericNumber", "GenericEvents<number>"),
        ("GenericUnknown", "GenericEvents<unknown>"),
        ("GenericNever", "GenericEvents<never>"),
    ];
    for (name, shape) in shapes {
        source.push_str(&format!(
            "type Shape{name} = {shape};\n\
             type Options{name} = Assert<Equal<__EmitOptions<Shape{name}>, __LegacyEmitOptions<Shape{name}>>>;\n\
             type Props{name} = Assert<Equal<__EmitProps<Shape{name}>, __LegacyEmitProps<Shape{name}>>>;\n\
             type Parameters{name} = Assert<Equal<$OverloadParameters<__EmitFn<Shape{name}>>, $LegacyOverloadParameters<__EmitFn<Shape{name}>>>>;\n"
        ));
    }
    source.push_str(
        r#"
function freePayload<T>(actual: __EmitOptions<GenericEvents<T>>,
  legacy: __LegacyEmitOptions<GenericEvents<T>>) {
  actual = legacy;
  legacy = actual;
}
type DistinctPayloads = Assert<Equal<Equal<
  __EmitOptions<{ done: [value: number] }>,
  __EmitOptions<{ done: [value: string] }>>, false>>;
type DistinctNames = Assert<Equal<Equal<
  __EmitOptions<{ left: [value: number] }>,
  __EmitOptions<{ right: [value: number] }>>, false>>;
"#,
    );
    for total in [16, 17, 100] {
        let model_members = (0..total - 1)
            .map(|index| format!("'update:m{index}': [value: number]"))
            .collect::<Vec<_>>()
            .join("; ");
        let names = std::iter::once("'e0'".to_owned())
            .chain((0..total - 1).map(|index| format!("'update:m{index}'")))
            .collect::<Vec<_>>()
            .join(" | ");
        source.push_str(&format!(
            "type Events{total} = __EmitFn<{{ e0: [value: number] }}> & __EmitFn<{{ {model_members} }}>;\n\
             type Names{total} = {names};\n\
             type Keys{total} = Assert<Equal<keyof __EmitOptions<Events{total}>, Names{total}>>;\n\
             type HandlerKeys{total} = Assert<Equal<keyof __EmitProps<Events{total}>, $HandlerKey<Names{total}> | `on${{Capitalize<Names{total}>}}`>>;\n\
             type Earliest{total} = Assert<Equal<Parameters<__EmitOptions<Events{total}>['e0']>, [value: number]>>;\n"
        ));
    }
    source
}

#[test]
fn emit_overload_tail_accumulator_preserves_shapes_and_all_events() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let source = emit_overload_contract_source();
    let project_root = create_project_case(
        "emit-overload-tail-accumulator-contract",
        &[("src/contract.ts", source.as_str())],
    );
    let valid = snapshot_project_diagnostics(&project_root)
        .expect("resolved native checker must return the complete valid contract diagnostics");
    println!("valid overload contract diagnostics: {valid:#?}");
    assert_eq!(valid, vec![]);

    let mut wrong = source.clone();
    wrong.push_str("declare const events: __EmitOptions<Events17>;\nevents.e0(\"wrong\");\n");
    std::fs::write(project_root.join("src/contract.ts"), &wrong).unwrap();
    let line = wrong[..wrong.find("events.e0(\"wrong\")").unwrap()]
        .bytes()
        .filter(|byte| *byte == b'\n')
        .count()
        + 1;
    let invalid = snapshot_project_diagnostics(&project_root)
        .expect("resolved native checker must return the complete wrong-payload diagnostics");
    println!("wrong overload payload diagnostics: {invalid:#?}");
    assert_eq!(
        invalid,
        vec![(
            "src/contract.ts".into(),
            Some(2345),
            format!(
                "{line}:11:error Argument of type 'string' is not assignable to parameter of type 'number'."
            )
            .into(),
        )]
    );

    let repaired = wrong.replace("events.e0(\"wrong\")", "events.e0(42)");
    assert_ne!(repaired, wrong);
    std::fs::write(project_root.join("src/contract.ts"), repaired).unwrap();
    let repair = snapshot_project_diagnostics(&project_root)
        .expect("resolved native checker must return the complete repaired contract diagnostics");
    println!("repaired overload payload diagnostics: {repair:#?}");
    assert_eq!(repair, vec![]);
    let _ = std::fs::remove_dir_all(&project_root);
}

// A generic overload never dedupes against its erased inference. Peeling one
// signature per step repeated it until depth 900 (~2.6M instantiations), and
// later stopped there, dropping every signature declared before it (#11435).
#[test]
fn generic_emit_overloads_keep_earlier_signatures_and_stay_bounded() {
    let Some(tsgo) = resolve_test_tsgo_binary() else {
        return;
    };
    let mut source = std::string::String::from("export {};\n");
    source.push_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/virtual_ts/helpers/declarations/core-types.d.ts.txt"
    )));
    source.push_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/virtual_ts/helpers/declarations/emit-overloads.d.ts.txt"
    )));
    source.push_str(
        r#"
type Equal<A, B> = (<T>() => T extends A ? 1 : 2) extends
  (<T>() => T extends B ? 1 : 2) ? true : false;
type Assert<T extends true> = T;
type Info = { user: { payload: number }; tag: { payload: string } };
type Stalled = {
  <T extends keyof Info>(event: 'done', value: { type: T; value: Info[T]['payload'] }): void;
  (event: 'closed'): void;
};
type Names = Assert<Equal<$OverloadParameters<Stalled>[0], 'done' | 'closed'>>;
type Mixed = { (event: 'a', x: number): void; <U extends string>(event: 'b', u: U): void; (event: 'c'): void };
type MixedNames = Assert<Equal<$OverloadParameters<Mixed>[0], 'a' | 'b' | 'c'>>;
type Plain = { (event: 'a', x: number): void; (event: 'a', x?: number): void; (event: 'b'): void };
type PlainNames = Assert<Equal<$OverloadParameters<Plain>[0], 'a' | 'b'>>;
type Issue = { (e: 'first', v: number): void; <T>(e: 'gen', v: T): void; (e: 'last', v: string): void };
type IssueArgs = Assert<Equal<$OverloadParameters<Issue>, [e: 'first', v: number] | [e: 'gen', v: unknown] | [e: 'last', v: string]>>;
type TwoGeneric = { (e: 'a'): void; <T>(e: 'b', v: T): void; <U extends string>(e: 'c', u: U): void; (e: 'd'): void };
type TwoGenericNames = Assert<Equal<$OverloadParameters<TwoGeneric>[0], 'a' | 'b' | 'c' | 'd'>>;
type Twenty = { (e: 'p0'): void; (e: 'p1'): void; (e: 'p2'): void; (e: 'p3'): void; (e: 'p4'): void; (e: 'p5'): void; (e: 'p6'): void; (e: 'p7'): void; (e: 'p8'): void; (e: 'p9'): void; (e: 'p10'): void; (e: 'p11'): void; (e: 'p12'): void; (e: 'p13'): void; (e: 'p14'): void; (e: 'p15'): void; (e: 'p16'): void; (e: 'p17'): void; (e: 'p18'): void; (e: 'p19'): void; <T>(e: 'g', v: T): void };
type TwentyNames = Assert<Equal<$OverloadParameters<Twenty>[0], 'p0' | 'p1' | 'p2' | 'p3' | 'p4' | 'p5' | 'p6' | 'p7' | 'p8' | 'p9' | 'p10' | 'p11' | 'p12' | 'p13' | 'p14' | 'p15' | 'p16' | 'p17' | 'p18' | 'p19' | 'g'>>;
"#,
    );
    let project_root = create_project_case(
        "generic-emit-overload-stall",
        &[("src/stall.ts", source.as_str())],
    );
    let output = std::process::Command::new(tsgo)
        .current_dir(&project_root)
        .args([
            "--ignoreConfig",
            "--noEmit",
            "--strict",
            "--extendedDiagnostics",
            "src/stall.ts",
        ])
        .output()
        .unwrap();
    let stdout = std::string::String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "{stdout}");
    let instantiations = stdout
        .lines()
        .find_map(|line| line.strip_prefix("Instantiations:"))
        .and_then(|value| value.trim().parse::<u64>().ok())
        .unwrap();
    assert!(instantiations < 200_000, "{stdout}");
    let _ = std::fs::remove_dir_all(&project_root);
}
