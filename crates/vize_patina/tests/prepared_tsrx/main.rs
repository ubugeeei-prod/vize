#![expect(
    clippy::unwrap_used,
    reason = "the input manifest and original committed outputs are test fixtures"
)]

use std::{fs, path::Path};
use vize_l2::lang::tsrx::prepared::{RegionLowering, with_prepared};
use vize_l4::lang::tsrx::{CompileOptions, Compiled, Failure, Target, compile, compile_prepared};
use vize_patina::{HelpLevel, LintPreset, LintResult, Linter, Locale};

mod profile;

type Products = (LintResult, Result<Compiled, Failure>);

fn dom_options(filename: &str) -> CompileOptions<'_> {
    CompileOptions {
        filename,
        target: Target::Dom,
        source_map: false,
        erase_types: false,
    }
}

fn original(linter: &Linter, source: &str, filename: &str) -> Products {
    let lint = linter.lint_tsrx(source, filename);
    let compiled = compile(source, &dom_options(filename));
    (lint, compiled)
}

fn retained(linter: &Linter, source: &str, filename: &str) -> Products {
    with_prepared(source, |stages| {
        let lint = linter.lint_prepared_tsrx(&stages, filename);
        (lint, compile_prepared(stages, &dom_options(filename)))
    })
}

fn same(before: &Products, after: &Products, filename: &str) {
    // LintResult's derived Debug includes the full labels, optional help, fix
    // edits, severities and both counts. The compiler result has structural Eq.
    assert_eq!(
        format!("{:?}", before.0),
        format!("{:?}", after.0),
        "{filename}"
    );
    assert_eq!(&before.1, &after.1, "{filename}");
}

fn each_file(mut inspect: impl FnMut(&Path, &str)) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let paths = fs::read(std::env::var_os("VIZE_TSRX_INPUTS").unwrap()).unwrap();
    for entry in paths.split(|b| *b == 0).filter(|s| !s.is_empty()) {
        let path = root.join(std::str::from_utf8(entry).unwrap());
        let text = fs::read_to_string(&path).unwrap();
        inspect(&path, &text);
    }
}

#[test]
fn all_readers_observe_the_same_host_expressions_and_original_product_refusals() {
    let cases = [
        "",
        "export const x = 1;",
        "export function Card({x}: {x: string}) @{ <img  src={x}/> }",
        "export function List({xs}: {xs: string[]}) @{ <ul>{xs.map(x => <li>{x}</li>)}</ul> }",
        "export function Map() @{ <div>&amp;</div> }",
        "export function Bad() @{ <a></b> }",
        "export const outside = <b/>;",
    ];
    let mut kinds = [false; 3];
    for source in cases {
        with_prepared(source, |stages| {
            assert!(core::ptr::eq(stages.module().program().source_text, source));
            match stages.regions() {
                RegionLowering::Ready(artifacts) => {
                    assert!(core::ptr::eq(artifacts.module(), stages.module()));
                    kinds[0] = true;
                }
                RegionLowering::Refused(_) => kinds[1] = true,
                RegionLowering::Syntax => kinds[2] = true,
            }
            for locale in [Locale::En, Locale::Ja, Locale::Zh] {
                for preset in LintPreset::ALL {
                    let linter = Linter::with_preset(preset)
                        .with_locale(locale)
                        .with_help_level(HelpLevel::Full);
                    let actual = linter.lint_prepared_tsrx(&stages, "Same.tsrx");
                    let expected = linter.lint_tsrx(source, "Same.tsrx");
                    assert_eq!(format!("{actual:?}"), format!("{expected:?}"));
                }
            }
            assert_eq!(
                compile_prepared(stages, &dom_options("Same.tsrx")),
                original(&Linter::new(), source, "Same.tsrx").1
            );
        });
    }
    assert_eq!(kinds, [true; 3]);
}

#[test]
#[ignore = "requires the NUL-delimited original TSRX input manifest"]
fn original_modules_and_committed_mount_outputs() {
    let mut files = 0;
    let mut emitted = 0;
    let mut mounted = 0;
    let mut bytes = 0;
    let mut diagnostics = 0;
    let mut reports = 0;
    each_file(|path, source| {
        let filename = path.to_str().unwrap();
        let linter = Linter::with_preset(LintPreset::Opinionated).with_help_level(HelpLevel::Full);
        let before = original(&linter, source, filename);
        let actual = retained(&linter, source, filename);
        same(&before, &actual, filename);
        for target in [Target::Dom, Target::Ssr, Target::Vapor] {
            for source_map in [false, true] {
                for erase_types in [false, true] {
                    let options = CompileOptions {
                        filename,
                        target,
                        source_map,
                        erase_types,
                    };
                    let expected = compile(source, &options);
                    let observed =
                        with_prepared(source, |stages| compile_prepared(stages, &options));
                    assert_eq!(observed, expected, "{filename} {options:?}");
                }
            }
        }
        files += 1;
        reports += actual.0.diagnostics.len();
        match actual.1 {
            Ok(compiled) => {
                emitted += 1;
                bytes += compiled.code.len();
                let receipt = path.with_extension("dom.ts");
                if receipt.is_file() {
                    assert_eq!(compiled.code.as_bytes(), fs::read(receipt).unwrap());
                    mounted += 1;
                }
            }
            Err(failure) => diagnostics += failure.diagnostics.len(),
        }
    });
    assert!(emitted > 0 && files > emitted && mounted > 0 && reports > 0 && diagnostics > 0);
    eprintln!(
        "TSRX files={files} emitted={emitted} code_bytes={bytes} mounted={mounted} lint_reports={reports} compiler_diagnostics={diagnostics}"
    );
}
