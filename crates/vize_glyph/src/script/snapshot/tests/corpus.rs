//! Retain the original descriptors and parse each selected program once.
//! Compare both formatting entries under two complete option sets.
use super::*;
use std::{fs, path::Path};

#[test]
#[ignore = "requires a NUL-delimited manifest of original SFC or script paths"]
fn original_sources() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let list = fs::read(std::env::var_os("VIZE_RETAINED_FORMAT_MANIFEST").unwrap()).unwrap();
    let settings = [
        (FormatOptions::default(), None),
        (variation(), Some(ImportSortOptions::default())),
    ];
    let mut count = Counts::default();
    let (mut files, mut invalid, mut refused, mut scriptless) = (0, 0, 0, 0);
    for raw in list.split(|&b| b == 0).filter(|name| !name.is_empty()) {
        let name = core::str::from_utf8(raw).unwrap();
        let path = root.join(name);
        let bytes = fs::read(&path).unwrap_or_else(|error| panic!("{name}: {error}"));
        files += 1;
        let Ok(text) = core::str::from_utf8(&bytes) else {
            invalid += 1;
            continue;
        };
        let arena = Allocator::default();
        if name.ends_with(".vue") || name.ends_with(".vue.txt") {
            let descriptor = Vue.observe_descriptor(
                &arena,
                text,
                DescriptorOptions {
                    version: VueVersion::V3,
                    dialect: VueDialect::Vue,
                    template: SurfaceParseOptions::default(),
                },
            );
            let Ok(view) = descriptor.admitted() else {
                refused += 1;
                continue;
            };
            if view.ordinary().is_none() && view.setup().is_none() {
                scriptless += 1;
            }
            for script in [view.ordinary(), view.setup()].into_iter().flatten() {
                compare(
                    &arena,
                    script.block(),
                    ProgramOptions::module(script.lang()),
                    name,
                    &settings,
                    &mut count,
                );
            }
        } else {
            let ext = path.extension().and_then(|s| s.to_str()).unwrap();
            let lang = match ext {
                "ts" | "tsx" | "mts" | "cts" => Lang::Ts,
                "js" | "jsx" | "mjs" | "cjs" => Lang::Js,
                other => panic!("unexpected extension {other}"),
            };
            let goal = if matches!(ext, "cts" | "cjs") {
                ProgramGoal::Script
            } else {
                ProgramGoal::Module
            };
            let options = ProgramOptions {
                lang,
                goal,
                jsx: matches!(ext, "tsx" | "jsx"),
            };
            compare(
                &arena,
                SourceRoot::new(text).unwrap().whole_block(),
                options,
                name,
                &settings,
                &mut count,
            );
        }
    }
    assert!(count.scripts > 0 && count.outputs > 0);
    eprintln!(
        "files={files} invalid_utf8={invalid} descriptor_refused={refused} scriptless={scriptless} {count:?}"
    );
}
