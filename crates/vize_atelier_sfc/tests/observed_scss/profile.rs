use super::*;
use davinci_harness::alloc::measure;
use std::{hint::black_box, time::Instant};
use vize_l1::css::lang::StyleLanguage;
use vize_l2::lang::scss::ScssProgram;

#[test]
#[ignore = "paired timings and process-wide allocator require the isolated performance job"]
fn one_structure_serves_css_classes_and_definition_queries() {
    mark_installed();
    for (label, source) in [
        ("nested", "$s:3px;.a{padding:$s;.b{margin:$s}}".to_owned()),
        (
            "module",
            include_str!(
                "../../../../tests/_fixtures/_projects/style-preprocessors/src/CssModuleScss.vue"
            )
            .split_once("<style lang=\"scss\" module>")
            .unwrap()
            .1
            .split_once("</style>")
            .unwrap()
            .0
            .to_owned(),
        ),
        (
            "many rules",
            "$s:4px;".to_owned()
                + &(0..64)
                    .map(|i| format!(".c{i}{{width:$s}}"))
                    .collect::<String>(),
        ),
    ] {
        let block = whole(&source);
        let run = |joined| {
            let (program, classes) = if joined {
                let (style, program) = ScssProducts::read(block).into_parts();
                (program.unwrap(), style.into_authored_classes().unwrap())
            } else {
                let program = StyleLanguage::Scss
                    .with_original_structure(block, ScssProgram::lower)
                    .unwrap();
                (
                    program,
                    StyleLanguage::Scss
                        .authored_classes(block.source())
                        .unwrap(),
                )
            };
            let refs: Vec<_> = program
                .variable_uses()
                .iter()
                .map(|r| program.definition_at(r.span.start).unwrap().span)
                .collect();
            (vize_l4::targets::scss::emit(&program), classes, refs)
        };
        assert_eq!(run(false), run(true));
        let a = measure(|| drop(black_box(run(false)))).unwrap();
        let b = measure(|| drop(black_box(run(true)))).unwrap();
        let elapsed = |joined| {
            let start = Instant::now();
            for _ in 0..32 {
                drop(black_box(run(joined)));
            }
            start.elapsed().as_nanos() / 32
        };
        let (mut old, mut new) = (Vec::new(), Vec::new());
        for i in 0..35 {
            let first = i % 2 == 0;
            let x = elapsed(first);
            let y = elapsed(!first);
            if i >= 4 {
                let (a, b) = if first { (y, x) } else { (x, y) };
                old.push(a);
                new.push(b);
            }
        }
        old.sort_unstable();
        new.sort_unstable();
        eprintln!(
            "scss_structure case={label} ns={}/{} calls={}/{} peak={}/{}",
            old[15], new[15], a.calls, b.calls, a.peak_bytes_over_start, b.peak_bytes_over_start
        );
        assert!(
            new[15] <= old[15]
                && b.calls <= a.calls
                && b.peak_bytes_over_start <= a.peak_bytes_over_start
        );
    }
}
