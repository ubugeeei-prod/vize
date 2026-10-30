//! Native zero-call witnesses must preserve real overload results and NoAny.
//! These helpers serve direct `new` calls; reflection can observe the inert tail.
use super::super::{TSGO_ENV, check_project, output_text, workspace_root};
use std::path::PathBuf;

const HELPERS: &str = include_str!(
    "../../../../vize_canon/src/virtual_ts/helpers/declarations/slot-resolver.d.ts.txt"
);
const CONTROLS: &str = r#"export {};
type $IsAny<T> = 0 extends (1 & T) ? true:false;
type SlotsOf<I> = I extends {$slots:infer S}?S:never;
type RequireNotAny<T extends false> = T;
declare const Original_TwoFirstAny: {new():never;new(props:{value?:number}):{$slots:any}};
type C_TwoFirstAny = $SlotConstructor<typeof Original_TwoFirstAny>;
const Z_TwoFirstAny=new(undefined as unknown as $SlotConstructorWithoutArguments<C_TwoFirstAny>)();
const P_TwoFirstAny=new(undefined as unknown as $SlotConstructorWithArguments<C_TwoFirstAny>)({value:123},...([] as unknown as $SlotConstructorArgs<C_TwoFirstAny>));
const Result_TwoFirstAny=__vizeSlotConstructorInstance<C_TwoFirstAny>()(Z_TwoFirstAny,P_TwoFirstAny);
type AssertNoAny_TwoFirstAny=RequireNotAny<$IsAny<SlotsOf<typeof Result_TwoFirstAny>>>;
declare const Original_TwoLastUnknown: {new(props:{value?:number}):{$slots:unknown};new():never};
type C_TwoLastUnknown = $SlotConstructor<typeof Original_TwoLastUnknown>;
const Z_TwoLastUnknown=new(undefined as unknown as $SlotConstructorWithoutArguments<C_TwoLastUnknown>)();
const P_TwoLastUnknown=new(undefined as unknown as $SlotConstructorWithArguments<C_TwoLastUnknown>)({value:123},...([] as unknown as $SlotConstructorArgs<C_TwoLastUnknown>));
const Result_TwoLastUnknown=__vizeSlotConstructorInstance<C_TwoLastUnknown>()(Z_TwoLastUnknown,P_TwoLastUnknown);
type AssertNoAny_TwoLastUnknown=RequireNotAny<$IsAny<SlotsOf<typeof Result_TwoLastUnknown>>>;
declare const Original_NineFirstAny: {new():never;new(props:{kind:'brand0';value?:number}):{$slots:any};new(props:{kind:'brand1';value?:number}):{$slots:any};new(props:{kind:'brand2';value?:number}):{$slots:any};new(props:{kind:'brand3';value?:number}):{$slots:any};new(props:{kind:'brand4';value?:number}):{$slots:any};new(props:{kind:'brand5';value?:number}):{$slots:any};new(props:{kind:'brand6';value?:number}):{$slots:any};new(props:{kind:'brand7';value?:number}):{$slots:any};};
type C_NineFirstAny = $SlotConstructor<typeof Original_NineFirstAny>;
const Z_NineFirstAny=new(undefined as unknown as $SlotConstructorWithoutArguments<C_NineFirstAny>)();
const P_NineFirstAny=new(undefined as unknown as $SlotConstructorWithArguments<C_NineFirstAny>)({kind:'brand0' as const,value:123},...([] as unknown as $SlotConstructorArgs<C_NineFirstAny>));
const Result_NineFirstAny=__vizeSlotConstructorInstance<C_NineFirstAny>()(Z_NineFirstAny,P_NineFirstAny);
type AssertNoAny_NineFirstAny=RequireNotAny<$IsAny<SlotsOf<typeof Result_NineFirstAny>>>;
declare const Original_SeventeenFirstUnknown: {new():never;new(props:{kind:'brand0';value?:number}):{$slots:unknown};new(props:{kind:'brand1';value?:number}):{$slots:unknown};new(props:{kind:'brand2';value?:number}):{$slots:unknown};new(props:{kind:'brand3';value?:number}):{$slots:unknown};new(props:{kind:'brand4';value?:number}):{$slots:unknown};new(props:{kind:'brand5';value?:number}):{$slots:unknown};new(props:{kind:'brand6';value?:number}):{$slots:unknown};new(props:{kind:'brand7';value?:number}):{$slots:unknown};new(props:{kind:'brand8';value?:number}):{$slots:unknown};new(props:{kind:'brand9';value?:number}):{$slots:unknown};new(props:{kind:'brand10';value?:number}):{$slots:unknown};new(props:{kind:'brand11';value?:number}):{$slots:unknown};new(props:{kind:'brand12';value?:number}):{$slots:unknown};new(props:{kind:'brand13';value?:number}):{$slots:unknown};new(props:{kind:'brand14';value?:number}):{$slots:unknown};new(props:{kind:'brand15';value?:number}):{$slots:unknown};};
type C_SeventeenFirstUnknown = $SlotConstructor<typeof Original_SeventeenFirstUnknown>;
const Z_SeventeenFirstUnknown=new(undefined as unknown as $SlotConstructorWithoutArguments<C_SeventeenFirstUnknown>)();
const P_SeventeenFirstUnknown=new(undefined as unknown as $SlotConstructorWithArguments<C_SeventeenFirstUnknown>)({kind:'brand0' as const,value:123},...([] as unknown as $SlotConstructorArgs<C_SeventeenFirstUnknown>));
const Result_SeventeenFirstUnknown=__vizeSlotConstructorInstance<C_SeventeenFirstUnknown>()(Z_SeventeenFirstUnknown,P_SeventeenFirstUnknown);
type AssertNoAny_SeventeenFirstUnknown=RequireNotAny<$IsAny<SlotsOf<typeof Result_SeventeenFirstUnknown>>>;
declare const Original_NineLastAny: {new(props:{kind:'brand0';value?:number}):{$slots:any};new(props:{kind:'brand1';value?:number}):{$slots:any};new(props:{kind:'brand2';value?:number}):{$slots:any};new(props:{kind:'brand3';value?:number}):{$slots:any};new(props:{kind:'brand4';value?:number}):{$slots:any};new(props:{kind:'brand5';value?:number}):{$slots:any};new(props:{kind:'brand6';value?:number}):{$slots:any};new(props:{kind:'brand7';value?:number}):{$slots:any};new():never;};
type C_NineLastAny = $SlotConstructor<typeof Original_NineLastAny>;
const Z_NineLastAny=new(undefined as unknown as $SlotConstructorWithoutArguments<C_NineLastAny>)();
const P_NineLastAny=new(undefined as unknown as $SlotConstructorWithArguments<C_NineLastAny>)({kind:'brand0' as const,value:123},...([] as unknown as $SlotConstructorArgs<C_NineLastAny>));
const Result_NineLastAny=__vizeSlotConstructorInstance<C_NineLastAny>()(Z_NineLastAny,P_NineLastAny);
type AssertNoAny_NineLastAny=RequireNotAny<$IsAny<SlotsOf<typeof Result_NineLastAny>>>;
declare const Original_DefaultZero: new<T=string>()=>{$slots:{default:(props:{value:T})=>void}};
type C_DefaultZero = $SlotConstructor<typeof Original_DefaultZero>;
const Z_DefaultZero=new(undefined as unknown as $SlotConstructorWithoutArguments<C_DefaultZero>)();
const P_DefaultZero=new(undefined as unknown as $SlotConstructorWithArguments<C_DefaultZero>)({value:123},...([] as unknown as $SlotConstructorArgs<C_DefaultZero>));
const Result_DefaultZero=__vizeSlotConstructorInstance<C_DefaultZero>()(Z_DefaultZero,P_DefaultZero);
type AssertNoAny_DefaultZero=RequireNotAny<$IsAny<SlotsOf<typeof Result_DefaultZero>>>;
declare const Original_MixedGenericNeverFirst: {new<T=never>():T;new(props:{value?:number}):{$slots:any}};
type C_MixedGenericNeverFirst = $SlotConstructor<typeof Original_MixedGenericNeverFirst>;
const Z_MixedGenericNeverFirst=new(undefined as unknown as $SlotConstructorWithoutArguments<C_MixedGenericNeverFirst>)();
const P_MixedGenericNeverFirst=new(undefined as unknown as $SlotConstructorWithArguments<C_MixedGenericNeverFirst>)({value:123},...([] as unknown as $SlotConstructorArgs<C_MixedGenericNeverFirst>));
const Result_MixedGenericNeverFirst=__vizeSlotConstructorInstance<C_MixedGenericNeverFirst>()(Z_MixedGenericNeverFirst,P_MixedGenericNeverFirst);
type AssertNoAny_MixedGenericNeverFirst=RequireNotAny<$IsAny<SlotsOf<typeof Result_MixedGenericNeverFirst>>>;
declare const Original_MixedGenericNeverLast: {new(props:{value?:number}):{$slots:any};new<T=never>():T};
type C_MixedGenericNeverLast = $SlotConstructor<typeof Original_MixedGenericNeverLast>;
const Z_MixedGenericNeverLast=new(undefined as unknown as $SlotConstructorWithoutArguments<C_MixedGenericNeverLast>)();
const P_MixedGenericNeverLast=new(undefined as unknown as $SlotConstructorWithArguments<C_MixedGenericNeverLast>)({value:123},...([] as unknown as $SlotConstructorArgs<C_MixedGenericNeverLast>));
const Result_MixedGenericNeverLast=__vizeSlotConstructorInstance<C_MixedGenericNeverLast>()(Z_MixedGenericNeverLast,P_MixedGenericNeverLast);
type AssertNoAny_MixedGenericNeverLast=RequireNotAny<$IsAny<SlotsOf<typeof Result_MixedGenericNeverLast>>>;
Result_DefaultZero.$slots.default({value:123});
"#;

#[test]
fn zero_witnesses_preserve_native_overload_results_and_public_exports() {
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping native zero-witness laws: {TSGO_ENV} is not set");
        return;
    };
    assert!(tsgo.is_file(), "{TSGO_ENV} must name an executable");
    let cases_root = workspace_root().join("target/vize-tests/tests");
    std::fs::create_dir_all(&cases_root).unwrap();
    let receipt = tempfile::Builder::new()
        .prefix("content-mapper-native-zero-witness-")
        .tempdir_in(&cases_root)
        .unwrap();
    let declaration = HELPERS
        .lines()
        .find(|line| line.starts_with("type $SlotConstructorWithoutArguments<C> ="))
        .unwrap();
    assert_eq!(HELPERS.matches(declaration).count(), 1);
    for (mode, extension) in [("embedded", "ts"), ("esm", "mts"), ("commonjs", "cts")] {
        let project = receipt.path().join(mode);
        std::fs::create_dir_all(&project).unwrap();
        let helpers = if mode == "embedded" {
            HELPERS.to_owned()
        } else {
            let package = project.join("node_modules/vize");
            std::fs::create_dir_all(&package).unwrap();
            std::fs::write(
                package.join("package.json"),
                include_str!("../../../../../npm/cli/package.json"),
            )
            .unwrap();
            for (name, contents) in [
                (
                    "virtual-ts.d.ts",
                    include_str!("../../../../../npm/cli/virtual-ts.d.ts"),
                ),
                (
                    "virtual-ts.d.mts",
                    include_str!("../../../../../npm/cli/virtual-ts.d.mts"),
                ),
                (
                    "virtual-ts.d.cts",
                    include_str!("../../../../../npm/cli/virtual-ts.d.cts"),
                ),
            ] {
                std::fs::write(package.join(name), contents).unwrap();
            }
            HELPERS.replace(declaration, "type $SlotConstructorWithoutArguments<C> = import(\"vize/virtual-ts\").$SlotConstructorWithoutArguments<C>;")
        };
        let source = format!("{helpers}\n{CONTROLS}");
        let bad = "Result_DefaultZero.$slots.default({value:123});";
        assert_eq!(source.matches(bad).count(), 1);
        let line = source
            .split_once(bad)
            .unwrap()
            .0
            .bytes()
            .filter(|byte| *byte == b'\n')
            .count()
            + 1;
        let column = bad.find("value").unwrap() + 1;
        let file = format!("proof.{extension}");
        std::fs::write(project.join(&file), &source).unwrap();
        std::fs::write(project.join("tsconfig.json"), serde_json::to_vec_pretty(&serde_json::json!({
            "compilerOptions": {
                "strict": true, "noEmit": true, "skipLibCheck": true,
                "module": "Node16", "moduleResolution": "Node16", "target": "ESNext", "types": [],
            },
            "files": [file],
        })).unwrap()).unwrap();
        let output = check_project(&tsgo, &project, "tsconfig.json");
        assert_eq!(
            output.status.code(),
            Some(2),
            "{mode}: {}",
            output_text(&output)
        );
        assert_eq!(output.stderr, b"", "{mode}: {}", output_text(&output));
        assert_eq!(
            std::str::from_utf8(&output.stdout).unwrap(),
            format!(
                "{file}({line},{column}): error TS2322: Type 'number' is not assignable to type 'string'.\n"
            ),
            "{mode}: {}",
            output_text(&output)
        );
        std::fs::write(
            project.join(file),
            source.replace(bad, "Result_DefaultZero.$slots.default({value:\"valid\"});"),
        )
        .unwrap();
        super::assert_valid(&check_project(&tsgo, &project, "tsconfig.json"));
    }
}
