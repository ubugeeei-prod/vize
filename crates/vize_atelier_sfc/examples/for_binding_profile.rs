//! Isolated original-goal parser counters; no timing-based acceptance claim.
use serde_json::json;
use vize_atelier_sfc::{SfcCompileOptions, SfcParseOptions, compile_sfc, parse_sfc};
use vize_l0::profiler::global_profiler;

fn main() {
    let profiler = global_profiler();
    let mut rows = Vec::new();
    for (name, template) in [
        ("ordinary", "<p v-for='row of items'>{{ row }}</p>"),
        (
            "ordinary-pattern",
            "<p v-for='{ label: row } of items'>{{ row }}</p>",
        ),
        ("escaped", r"<p v-for='\u005frow of items'>{{ _row }}</p>"),
        (
            "escaped-parameter-default",
            r#"<p v-for="\u005frow = 'A' of items">{{ _row }}</p>"#,
        ),
        (
            "escaped-pattern",
            r"<p v-for='{ label: \u005frow = fallback } of items'>{{ _row }}</p>",
        ),
        (
            "escaped-key-index",
            r"<p v-for='(row, \u005fkey, \u005findex) in items'>{{ _key }}{{ _index }}</p>",
        ),
        (
            "ordinary-key-literal",
            r#"<p v-for="row of items" :key="capture('_for_item0.value')">ok</p>"#,
        ),
        (
            "escaped-key-literal",
            r#"<p v-for="\u005frow of items" :key="capture('_for_item0.value')">ok</p>"#,
        ),
        (
            "escaped-key-callback-shadow",
            r#"<p v-for="\u005frow of items" :key="(_for_item0 => capture(_for_item0.value))({value: 'shadow'})">ok</p>"#,
        ),
    ] {
        let source = [
            "<script>export default {}</script><template>",
            template,
            "</template>",
        ]
        .concat();
        for backend in ["dom", "ssr", "vapor"] {
            let Ok(descriptor) = parse_sfc(&source, SfcParseOptions::default()) else {
                std::process::exit(1);
            };
            let mut options = SfcCompileOptions::default();
            options.vapor = backend == "vapor";
            options.template.ssr = backend == "ssr";
            profiler.clear();
            profiler.enable();
            let result = compile_sfc(&descriptor, options);
            profiler.disable();
            let counters = profiler
                .counter_summary()
                .entries
                .into_iter()
                .map(|row| json!({"name":row.name,"samples":row.samples,"total":row.total}))
                .collect::<Vec<_>>();
            rows.push(json!({"case":name,"backend":backend,"source":source,"calls":1,"success":result.is_ok(),"counters":counters}));
        }
    }
    println!(
        "{}",
        json!({"sourceSha":std::env::var("SOURCE_SHA").unwrap_or_default(),"scope":"actual isolated full SFC compile calls; parser counters only, no speed ratio", "rows":rows})
    );
}
