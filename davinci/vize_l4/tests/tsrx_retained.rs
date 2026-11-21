#![cfg(feature = "retained-tsrx")]
use vize_l1_to_l2::lower::tsrx::with_prepared;
use vize_l4::lang::tsrx::{CompileOptions, Target, compile, compile_prepared};

#[test]
fn target_capabilities_and_stage_refusal_precedence_match_the_compiler() {
    for source in [
        "export function Card<T>({x}: {x: T}) @{ <b>{x}</b> }",
        "export const bad = <a></b>;",
        "export function Refused() @{ <p>one &amp; two</p> }",
        "export const Outside = <i />;",
    ] {
        for target in [Target::Dom, Target::Ssr, Target::Vapor] {
            for source_map in [false, true] {
                for erase_types in [false, true] {
                    let options = CompileOptions {
                        filename: "Original.tsrx",
                        target,
                        source_map,
                        erase_types,
                    };
                    let observed =
                        with_prepared(source, |stages| compile_prepared(stages, &options));
                    assert_eq!(observed, compile(source, &options), "{options:?}");
                }
            }
        }
    }
}
