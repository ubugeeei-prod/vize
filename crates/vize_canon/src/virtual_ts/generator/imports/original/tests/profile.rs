use super::*;
use davinci_harness::paired::assert_calibrated_paired_time;
use std::hint::black_box;

#[test]
#[ignore = "prepared and cold full-document pairs with exact heap gates"]
fn original_import_document_resources() {
    alloc::mark_installed();
    for count in [0, 1, 8, 32, 128] {
        let mut source = std::string::String::from("<!--🌱--><script setup lang='ts'>\n");
        for i in 0..count {
            source.push_str(&format!("import Value{i}, {{ type Props as Props{i}, value as local{i} }} from './item{i}';\n"));
        }
        source.push_str("const props = defineProps<{title: string}>();\n</script><template><span>{{props.title}}</span></template>");
        let arena = Allocator::new();
        let descriptor = Vue.observe_descriptor(&arena, &source, options());
        let selected = descriptor.admitted().unwrap();
        let block = selected.setup().unwrap().block();
        let original = AuthoredProgram::parse(&arena, block, SourceType::ts());
        let facts = OriginalImportFacts::new(&original).unwrap();
        let template = selected
            .template()
            .map(|template| vize_armature::parse(&arena, template.block().source()).0);
        let summary = analyze(block.source(), true, template.as_ref());
        let options = VirtualTsOptions::default();
        let template_offset = selected
            .template()
            .map_or(0, |template| template.block().start());
        let generate = |joined, facts: &OriginalImportFacts<'_>| {
            if joined {
                generate_virtual_ts_with_original_imports(
                    black_box(&summary),
                    black_box(block.source()),
                    template.as_ref(),
                    block.start(),
                    template_offset,
                    &options,
                    black_box(facts),
                )
                .unwrap()
            } else {
                generate_virtual_ts_with_offsets(
                    black_box(&summary),
                    Some(black_box(block.source())),
                    template.as_ref(),
                    block.start(),
                    template_offset,
                    &options,
                )
            }
        };
        same(
            &generate(true, &facts),
            &generate(false, &facts),
            "prepared",
        );
        for cold in [false, true] {
            let run = |joined| {
                if cold {
                    // Both arms perform the same original Program observation;
                    // the candidate additionally constructs its import demand.
                    let arena = Allocator::new();
                    let original =
                        AuthoredProgram::parse(&arena, black_box(block), SourceType::ts());
                    if joined {
                        let facts = OriginalImportFacts::new(black_box(&original)).unwrap();
                        black_box(generate(true, &facts));
                    } else {
                        black_box(&original);
                        black_box(generate(false, &facts));
                    }
                } else {
                    black_box(generate(joined, &facts));
                }
            };
            let old = alloc::measure(|| run(false)).unwrap();
            let new = alloc::measure(|| run(true)).unwrap();
            eprintln!(
                "original_import_heap count={count} cold={cold} calls={}/{} peak={}/{}",
                old.calls, new.calls, old.peak_bytes_over_start, new.peak_bytes_over_start
            );
            assert!(
                new.calls <= old.calls && new.peak_bytes_over_start <= old.peak_bytes_over_start
            );
            assert_calibrated_paired_time(
                &format!("original imports count={count} cold={cold}"),
                8,
                run,
            );
        }
    }
}
