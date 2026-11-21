//! Counterbalanced individual requests on original scripts across the corpus.
use super::*;
use std::{fs, path::Path};

fn measure(text: &str, language: ProgramOptions, label: &str) -> bool {
    let arena = Allocator::default();
    let block = SourceRoot::new(text).unwrap().whole_block();
    let program = parse_program_once(
        &arena,
        EmbedSource::authored(text, block.span()).unwrap(),
        language,
    );
    let input = Input::new(&arena, &program);
    let options = FormatOptions::default();
    let sort = ImportSortOptions::default();
    let control = Allocator::default();
    use crate::format_script_with_sort_imports;
    if format_script_with_sort_imports(text, &options, &control, language.source_type(), None)
        .is_err()
        || format_script_with_sort_imports(
            text,
            &variation(),
            &control,
            language.source_type(),
            Some(&sort),
        )
        .is_err()
    {
        eprintln!("format refusal {label}");
        return false;
    }
    for demand in [
        Demand::First,
        Demand::FirstBorrow,
        Demand::Repeated,
        Demand::Both,
        Demand::Different,
    ] {
        let a = window(&input, demand, false, false);
        let b = window(&input, demand, true, false);
        assert_eq!(a.0, b.0);
        let (base, head) = (a.2.unwrap(), b.2.unwrap());
        assert!(head.calls <= base.calls, "{label} {demand:?}");
        assert!(
            head.peak_bytes_over_start <= base.peak_bytes_over_start,
            "{label} {demand:?}: {head:?} vs {base:?}"
        );
        if matches!(demand, Demand::Repeated) {
            assert_eq!((head.calls, head.peak_bytes_over_start), (0, 0));
        }
        let mut samples = Vec::new();
        for i in 0..4 {
            let reverse = i % 2 == 1;
            let a = window(&input, demand, reverse, true);
            let b = window(&input, demand, !reverse, true);
            samples.push(if reverse { (b.1, a.1) } else { (a.1, b.1) });
        }
        eprintln!(
            "original demand={demand:?} bytes={} calls={}/{} peak={}/{} pairs={samples:?} source={label}",
            text.len(),
            base.calls,
            head.calls,
            base.peak_bytes_over_start,
            head.peak_bytes_over_start
        );
    }
    true
}

#[test]
#[ignore = "stratified NUL-delimited original source manifest for timing and allocation windows"]
fn original_formatter_requests() {
    alloc::mark_installed();
    let list = fs::read(std::env::var_os("VIZE_RETAINED_FORMAT_SAMPLE").unwrap()).unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut count = 0;
    let names = list
        .split(|&byte| byte == 0)
        .filter(|name| !name.is_empty());
    let size = names.clone().count();
    let stride = if size > 512 { size.div_ceil(256) } else { 1 };
    for name in names.step_by(stride) {
        let path = core::str::from_utf8(name).unwrap();
        let bytes = fs::read(root.join(path)).unwrap();
        let Ok(source) = core::str::from_utf8(&bytes) else {
            eprintln!("non UTF-8 {path}");
            continue;
        };
        if path.ends_with(".vue") || path.ends_with(".vue.txt") {
            let arena = Allocator::default();
            let descriptor = Vue.observe_descriptor(
                &arena,
                source,
                DescriptorOptions {
                    version: VueVersion::V3,
                    dialect: VueDialect::Vue,
                    template: SurfaceParseOptions::default(),
                },
            );
            let Ok(view) = descriptor.admitted() else {
                continue;
            };
            for script in [view.ordinary(), view.setup()].into_iter().flatten() {
                count += usize::from(measure(
                    script.block().source(),
                    ProgramOptions::module(script.lang()),
                    path,
                ));
            }
        } else {
            let extension = Path::new(path).extension().unwrap().to_str().unwrap();
            let lang = match extension {
                "js" | "jsx" | "cjs" | "mjs" => Lang::Js,
                "ts" | "tsx" | "cts" | "mts" => Lang::Ts,
                other => panic!("{other}"),
            };
            let goal = if matches!(extension, "cjs" | "cts") {
                ProgramGoal::Script
            } else {
                ProgramGoal::Module
            };
            count += usize::from(measure(
                source,
                ProgramOptions {
                    lang,
                    jsx: matches!(extension, "jsx" | "tsx"),
                    goal,
                },
                path,
            ));
        }
    }
    eprintln!("profiled {count} original scripts");
    assert!(count > 0);
}
