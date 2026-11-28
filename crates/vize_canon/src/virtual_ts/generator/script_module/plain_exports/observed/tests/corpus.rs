use super::*;
use std::{fs, path::Path};

#[test]
#[ignore = "compare the full contents of plain-script bridges in original Vue documents"]
fn original_documents() {
    run(false);
}

#[test]
#[ignore = "per-document cold and retained timings with an isolated allocation window"]
fn original_performance() {
    alloc::mark_installed();
    run(true);
}

fn run(profile: bool) {
    let manifest = fs::read_to_string(std::env::var_os("VIZE_BRIDGE_INPUTS").unwrap()).unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let (mut inputs, mut invalid, mut containers, mut other, mut external, mut grammar, mut syntax) =
        (0, 0, 0, 0, 0, 0, 0);
    let (mut complete, mut retries, mut keywords, mut hoists, mut bytes, mut rows, mut links) =
        (0, 0, 0, 0, 0, 0, 0);
    let mut regressions = Vec::new();
    for name in manifest.split_terminator('\0') {
        inputs += 1;
        let data = fs::read(root.join(name)).unwrap();
        let Ok(source) = std::str::from_utf8(&data) else {
            invalid += 1;
            continue;
        };
        let Ok(descriptor) = parse_sfc(source, Default::default()) else {
            containers += 1;
            continue;
        };
        if descriptor.script.is_none() || descriptor.script_setup.is_some() {
            other += 1;
            continue;
        }
        let block = frame(&descriptor);
        let arena = Allocator::new();
        let (ts, retry) = parses(&arena, block);
        let plan = match PlainScriptBridge::new(&descriptor, &ts, retry.as_ref()) {
            Ok(plan) => plan,
            Err(PlainScriptBridgeError::ExternalScript) => {
                external += 1;
                continue;
            }
            Err(PlainScriptBridgeError::Grammar) => {
                grammar += 1;
                continue;
            }
            Err(PlainScriptBridgeError::Syntax(_)) => {
                syntax += 1;
                continue;
            }
            Err(other) => panic!("{name}: {other:?}"),
        };
        let summary = analyze_sfc_descriptor(&descriptor, None, SfcCroquisOptions::full());
        compare_plans(&plan, &previous(None, block), &summary);
        if profile {
            for cold in [false, true] {
                if !super::profile::measure(
                    &descriptor,
                    &summary,
                    (&ts, retry.as_ref()),
                    name,
                    cold,
                    1,
                ) {
                    regressions.push((name, cold));
                }
            }
        } else {
            let (b, r, l) = public_footer(&plan, &summary);
            bytes += b;
            rows += r;
            links += l;
        }
        complete += 1;
        retries += usize::from(retry.is_some());
        keywords += plan.export_keywords().count();
        hoists += plan.relocated_declarations().count();
    }
    assert_eq!(
        inputs,
        invalid + containers + other + external + grammar + syntax + complete
    );
    assert!(complete > 0);
    eprintln!(
        "bridge_results inputs={inputs} invalid={invalid} containers={containers} other={other} external={external} grammar={grammar} syntax={syntax} complete={complete} retries={retries} keywords={keywords} hoists={hoists} bytes={bytes} mappings={rows} links={links}"
    );
    assert!(regressions.is_empty(), "{regressions:?}");
}
