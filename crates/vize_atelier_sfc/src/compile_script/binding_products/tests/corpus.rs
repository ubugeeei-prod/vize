use super::*;
use std::{fs, path::Path};

#[test]
#[ignore = "compare macro-shadow sets and public project effect graphs on original SFCs"]
fn original_script_products() {
    run(false);
}

#[test]
#[ignore = "cold and retained request cost for every original selected caller"]
fn original_performance() {
    alloc::mark_installed();
    run(true);
}

fn run(profile: bool) {
    let mut regressions = Vec::new();
    let manifest =
        fs::read_to_string(std::env::var_os("VIZE_SCRIPT_QUERY_INPUTS").unwrap()).unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let (mut inputs, mut invalid, mut containers, mut absent, mut syntax, mut complete) =
        (0, 0, 0, 0, 0, 0);
    let (mut scripts, mut names, mut edges, mut cycles) = (0, 0, 0, 0);
    let (mut external, mut without_file) = (0, 0);
    for filename in manifest.split_terminator('\0') {
        inputs += 1;
        let bytes = fs::read(root.join(filename)).unwrap();
        let Ok(source) = std::str::from_utf8(&bytes) else {
            invalid += 1;
            continue;
        };
        let Ok(descriptor) = crate::parse_sfc(source, Default::default()) else {
            containers += 1;
            continue;
        };
        let blocks = blocks(source, &descriptor);
        if blocks.iter().all(Option::is_none) {
            absent += 1;
            continue;
        }
        let arena = Allocator::new();
        let parsed = parse(&arena, blocks);
        let result = read(&descriptor, &parsed);
        if descriptor
            .script
            .iter()
            .chain(&descriptor.script_setup)
            .any(|b| b.src.is_some())
        {
            // A previously selected source block can have a syntax issue first.
            assert!(
                matches!(
                    result,
                    Err(ScriptBindingError::ExternalScript
                        | ScriptBindingError::Effect(ObservedEffectError::Syntax(_)))
                ),
                "{filename}"
            );
            external += 1;
            continue;
        }
        if parsed
            .iter()
            .flatten()
            .any(|p| p.parsed().panicked || !p.parsed().diagnostics.is_empty())
        {
            assert!(
                matches!(
                    result,
                    Err(ScriptBindingError::Effect(ObservedEffectError::Syntax(_)))
                ),
                "{filename}"
            );
            syntax += 1;
            continue;
        }
        let actual = result.unwrap();
        let expected = original(blocks, &parsed);
        compare(&actual, &expected, filename);
        if profile {
            for cold in [false, true] {
                if !super::profile::measure(&descriptor, blocks, &parsed, cold, filename, 1, false)
                {
                    regressions.push((filename, cold));
                }
            }
        }
        let observation = vize_l1_to_l2::native_file::lower_sfc_native(
            &arena,
            source,
            vize_l1::container::vue::DescriptorOptions {
                version: vize_carton::config::VueVersion::V3,
                dialect: vize_carton::config::VueDialect::Vue,
                template: Default::default(),
            },
        );
        without_file += usize::from(observation.admitted().is_none());
        complete += 1;
        scripts += parsed.iter().flatten().count();
        names += actual.setup.len() + actual.ordinary.len();
        edges += actual.effects.edges().count();
        cycles += actual.effects.summary().cycle_count;
    }
    assert_eq!(
        inputs,
        invalid + containers + absent + external + syntax + complete
    );
    assert!(scripts > 0 && names > 0 && edges > 0);
    eprintln!(
        "script_effect_results inputs={inputs} invalid={invalid} containers={containers} absent={absent} external={external} syntax={syntax} without_file={without_file} complete={complete} blocks={scripts} names={names} edges={edges} cycles={cycles}"
    );
    assert!(regressions.is_empty(), "{regressions:?}");
}
