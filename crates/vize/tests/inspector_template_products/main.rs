//! Complete serialized DevTools payloads together with Doctor's typed rows.
#![expect(
    clippy::unwrap_used,
    reason = "fixed serialized test products must be available"
)]
use davinci_harness::alloc;
use std::{fs, hint::black_box, path::Path, time::Instant};
use vize_croquis_cf::TemplateComplexity;
use vize_curator::inspector::{
    InspectorOptions, InspectorSourceFile, InspectorTarget, InspectorTemplateSyntax, build_payload,
    build_payload_with_complexity, serialize_payload,
};
use vize_l0::String;

#[global_allocator]
static ALLOCATOR: alloc::CountingAllocator<std::alloc::System> = alloc::CountingAllocator::system();

fn read(
    files: &[InspectorSourceFile],
    target: InspectorTarget,
    options: InspectorOptions,
    shared: bool,
) -> (String, Vec<Option<TemplateComplexity>>) {
    let mut components = Vec::with_capacity(files.len());
    let payload = if shared {
        build_payload_with_complexity(target, options, files.to_vec(), |_, entry| {
            components.push(entry)
        })
    } else {
        let payload = build_payload(target, options, files.to_vec());
        components.extend(files.iter().map(|file| {
            if file.path.ends_with(".vue") {
                TemplateComplexity::from_sfc(&file.source)
            } else {
                None
            }
        }));
        payload
    };
    (serialize_payload(&payload).unwrap(), components)
}

fn file(path: &str, source: &str) -> InspectorSourceFile {
    InspectorSourceFile {
        path: path.into(),
        source: source.into(),
    }
}

#[test]
fn serialized_inspector_options_and_selections_match() {
    let files = [
        file("index.ts", "export {}"),
        file(
            "child.vue",
            "<script setup>const a=1</script>\r\n<template><b v-if='a'>猫</b></template>",
        ),
    ];
    for target in [
        InspectorTarget::Dom,
        InspectorTarget::Ssr,
        InspectorTarget::Vapor,
    ] {
        for template_syntax in [
            InspectorTemplateSyntax::Standard,
            InspectorTemplateSyntax::Strict,
            InspectorTemplateSyntax::Quirks,
        ] {
            for custom_renderer in [false, true] {
                let options = InspectorOptions {
                    custom_renderer,
                    template_syntax,
                };
                for docs in [&[][..], &files[..], &files[1..]] {
                    assert_eq!(
                        read(docs, target, options, true),
                        read(docs, target, options, false)
                    );
                }
            }
        }
    }
}

#[test]
fn playground_share_url_and_full_agent_report_accept_the_shared_payload() {
    use vize_curator::inspector::{
        build_agent_report, build_playground_url, serialize_agent_report,
    };
    let files = [
        file(
            "src/Parent.vue",
            "<script setup>import Child from './Child.vue'</script><template><Child/></template>",
        ),
        file(
            "src/Child.vue",
            "<script setup>const x=1</script><template><i v-if='x'>雪</i></template>",
        ),
    ];
    let options = InspectorOptions {
        custom_renderer: false,
        template_syntax: InspectorTemplateSyntax::Strict,
    };
    let report = |shared| {
        let payload = if shared {
            build_payload_with_complexity(InspectorTarget::Dom, options, files.to_vec(), |_, _| {})
        } else {
            build_payload(InspectorTarget::Dom, options, files.to_vec())
        };
        let json = serialize_payload(&payload).unwrap();
        let url = build_playground_url("https://vizejs.dev/play/?theme=light", &json);
        let report = build_agent_report(payload, url.clone(), files.to_vec());
        (url, serialize_agent_report(&report).unwrap())
    };
    assert_eq!(report(true), report(false));
}

fn paired(files: &[InspectorSourceFile], label: &str, n: u128) -> bool {
    let options = InspectorOptions {
        custom_renderer: false,
        template_syntax: InspectorTemplateSyntax::Standard,
    };
    let run = |shared| read(black_box(files), InspectorTarget::Dom, options, shared);
    assert_eq!(run(false), run(true), "{label}");
    let old = alloc::measure(|| run(false)).unwrap();
    let new = alloc::measure(|| run(true)).unwrap();
    let sample = |n, reverse| {
        let measure = |shared| {
            let t = Instant::now();
            for _ in 0..n {
                drop(black_box(run(shared)));
            }
            t.elapsed().as_nanos() / n
        };
        let mut values = Vec::with_capacity(31);
        for round in 0..35 {
            let first = (round % 2 == 0) != reverse;
            let a = measure(first);
            let b = measure(!first);
            if round >= 4 {
                values.push(if first { (b, a) } else { (a, b) });
            }
        }
        values
    };
    let mut timings = sample(n, false);
    let slower = |pairs: &[(u128, u128)]| pairs.iter().filter(|(a, b)| b > a).count() >= 23;
    let mut pass = new.calls <= old.calls && new.peak_bytes_over_start <= old.peak_bytes_over_start;
    if slower(&timings) {
        let confirm = sample(n * 8, true);
        eprintln!("inspector_confirm input={label} ns={confirm:?}");
        pass &= !slower(&confirm);
    }
    if n > 1 || !pass {
        eprintln!("inspector_pairs input={label} ns={timings:?}");
    }
    timings.sort_unstable_by_key(|p| p.0);
    let before = timings[15].0;
    timings.sort_unstable_by_key(|p| p.1);
    eprintln!(
        "inspector_request input={label} ns={}/{} calls={}/{} peak={}/{} pass={pass}",
        before,
        timings[15].1,
        old.calls,
        new.calls,
        old.peak_bytes_over_start,
        new.peak_bytes_over_start
    );
    pass
}

#[test]
#[ignore = "measure public Inspector serialization and complete Doctor components"]
fn complete_inspection_requests() {
    alloc::mark_installed();
    for count in [0, 1, 8, 32, 128] {
        let mut source = std::string::String::from(
            "<script setup>const rows=[]</script>\r\n<template><section>",
        );
        for _ in 0..count {
            source.push_str(
                "<Card v-for='row in rows'><i v-if='row.ready && row.key'>雪</i><b v-else/></Card>",
            );
        }
        source.push_str("</section></template>");
        let files = [file("App.vue", &source)];
        assert!(paired(&files, &count.to_string(), 4));
    }
}

#[test]
#[ignore = "measure the complete multi-file DevTools payload with its Doctor records"]
fn project_inspection_requests() {
    alloc::mark_installed();
    let sources = [
        file("src/App.vue", "<template><Child v-if='ok'/></template>"),
        file(
            "src/Child.vue",
            "<script setup>const rows=[]</script><template><b v-for='x in rows'>雪{{ x }}</b></template>",
        ),
        file("src/Other.vue", "<template lang='pug'>i はい</template>"),
        file("src/index.ts", "export {}"),
    ];
    for count in [0, 2, 4, 16, 64] {
        let files = sources
            .iter()
            .cycle()
            .take(count)
            .cloned()
            .collect::<Vec<_>>();
        assert!(paired(&files, &format!("project-{count}"), 16));
    }
}

#[test]
#[ignore = "measure individual original inspector and Doctor requests"]
fn original_inspection_requests() {
    alloc::mark_installed();
    let manifest =
        fs::read_to_string(std::env::var_os("VIZE_INSPECTOR_TEMPLATE_INPUTS").unwrap()).unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut failures = Vec::new();
    let mut count = 0;
    for name in manifest.split_terminator('\0') {
        let src = fs::read(root.join(name)).unwrap();
        let Ok(source) = std::str::from_utf8(&src) else {
            continue;
        };
        let logical = name.strip_suffix(".txt").unwrap_or(name);
        if !paired(&[file(logical, source)], logical, 1) {
            failures.push(name);
        }
        count += 1;
    }
    eprintln!("inspector_requests files={count}");
    assert!(count > 0);
    assert!(failures.is_empty(), "{failures:?}");
}
