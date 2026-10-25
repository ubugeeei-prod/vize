use serde_json::{Value, json};
use vize_carton::i18n::{Locale, translator};
use vize_l0::{
    Allocator, SourceRoot,
    config::{VueDialect, VueVersion},
    diag::{PartKind, Stage},
};
use vize_l1::{container::vue::DescriptorOptions, markup::SurfaceParseOptions};
use vize_l1_to_l2::native_file::lower_sfc_native;
use vize_l2::lang::js::{JsxFileProducer, JsxMarkupFile};
use vize_patina::{
    Linter, RuleRegistry,
    native::{NativeLintFinding, canonical::NativeCanonicalLint},
    rules::a11y::TabindexNoPositive,
};

#[derive(Clone, Copy)]
pub enum Dialect {
    Vue,
    Jsx,
    Tsx,
}

pub fn options() -> DescriptorOptions {
    DescriptorOptions {
        version: VueVersion::V3,
        dialect: VueDialect::Vue,
        template: SurfaceParseOptions::default(),
    }
}

pub fn jsx<'a>(a: &'a Allocator, source: &'a str, typescript: bool) -> JsxMarkupFile<'a> {
    let kind = if typescript {
        oxc_span::SourceType::tsx()
    } else {
        oxc_span::SourceType::jsx()
    }
    .with_module(true);
    let block = SourceRoot::new(source).unwrap().whole_block();
    let original = oxc_parser::Parser::new(a, source, kind).parse_observed();
    let mut producer = JsxFileProducer::new(a, original, block, 0)
        .unwrap_or_else(|rejected| panic!("native Program: {:?}", rejected.error()));
    producer.walk().unwrap();
    producer
        .finish_markup()
        .unwrap_or_else(|rejected| panic!("{:?}", rejected.issues()))
}

pub fn native(finding: &NativeLintFinding) -> Value {
    let d = finding.diagnostic();
    assert_eq!(d.stage, Stage::Surface);
    assert!(d.witness().is_none());
    assert_eq!(d.parts.len(), 1);
    let help = d.parts.first().unwrap();
    assert_eq!(help.kind, PartKind::Help);
    assert_eq!(help.span, d.span);
    json!({"rule":finding.rule_name(),"severity":d.severity().as_str(),"message":d.message.as_str(),"start":d.span.start,"end":d.span.end,"help":help.message.as_str(),"labels":[],"fix":null})
}

pub fn profile(lint: &NativeCanonicalLint<'_, '_>, lane: &str, expected: usize) {
    let messages = translator().for_locale(Locale::En);
    assert_eq!(
        lint.tabindex_no_positive(&messages).unwrap().len(),
        expected
    );
    let profiler = vize_l0::profiler::global_profiler();
    profiler.clear();
    profiler.enable();
    for _ in 0..500 {
        let findings = core::hint::black_box(lint.tabindex_no_positive(&messages).unwrap());
        assert_eq!(findings.len(), expected);
    }
    profiler.disable();
    let metric = profiler.get("patina.native.canonical.tabindex").unwrap();
    assert_eq!(metric.count, 500);
    assert_eq!(profiler.all().len(), 1);
    assert!(profiler.counter_snapshot().is_empty());
    eprintln!(
        "{}",
        json!({"lane":lane,"profile":"dev","nodes":lint.file().artifact().node_count(),"findings":expected,"calls":metric.count,"average_ns":metric.average().as_nanos(),"histogram_p95_ns":metric.percentile(0.95).as_nanos(),"min_ns":metric.min_duration.as_nanos(),"max_ns":metric.max_duration.as_nanos(),"scope":"query only, producer preparation and result destruction excluded"})
    );
}

fn query(
    lint: &NativeCanonicalLint<'_, '_>,
    messages: &impl vize_l0::diag::MessageLookup,
) -> Vec<NativeLintFinding> {
    struct Count<'m, M> {
        messages: &'m M,
        calls: core::cell::Cell<usize>,
    }
    impl<M: vize_l0::diag::MessageLookup> vize_l0::diag::MessageLookup for Count<'_, M> {
        fn lookup(&self, key: &str) -> std::borrow::Cow<'static, str> {
            self.calls.set(self.calls.get() + 1);
            self.messages.lookup(key)
        }
    }
    let counted = Count {
        messages,
        calls: core::cell::Cell::new(0),
    };
    let output = lint.tabindex_no_positive(&counted).unwrap();
    let lookups = if output.is_empty() { 0 } else { 2 };
    assert_eq!(
        counted.calls.get(),
        lookups,
        "catalog work is bounded per query, not per finding"
    );
    let repeated = lint.tabindex_no_positive(&counted).unwrap();
    assert_eq!(counted.calls.get(), lookups * 2);
    assert_eq!(
        output.iter().map(native).collect::<Vec<_>>(),
        repeated.iter().map(native).collect::<Vec<_>>()
    );
    output
}

pub fn parity(dialect: Dialect, body: &str, newline: &str, locale: Locale) -> Vec<Value> {
    let a = Allocator::default();
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(TabindexNoPositive));
    let linter = Linter::with_registry(registry).with_locale(locale);
    let messages = translator().for_locale(locale);
    let (source, actual, reference) = match dialect {
        Dialect::Vue => {
            let source = vize_l0::cstr!("<!--作者😀-->{newline}<template>{body}</template>");
            let owner = lower_sfc_native(&a, &source, options());
            let admitted = owner
                .admitted()
                .unwrap_or_else(|| panic!("{:?}", owner.issues()));
            let actual = NativeCanonicalLint::from_vue(admitted);
            assert!(core::ptr::eq(actual.file(), owner.file().unwrap().file()));
            let findings = query(&actual, &messages);
            let reference = linter.lint_sfc(&source, "native.vue");
            drop(owner);
            (source, findings, reference)
        }
        Dialect::Jsx | Dialect::Tsx => {
            let source = vize_l0::cstr!("const prefix = '作者😀';{newline}const view = {body};");
            let owner = jsx(&a, &source, matches!(dialect, Dialect::Tsx));
            let actual = NativeCanonicalLint::from_jsx(&owner).unwrap();
            assert!(core::ptr::eq(actual.file(), owner.file()));
            let findings = query(&actual, &messages);
            let lang = if matches!(dialect, Dialect::Tsx) {
                vize_atelier_jsx::JsxLang::Tsx
            } else {
                vize_atelier_jsx::JsxLang::Jsx
            };
            let reference = linter.lint_jsx(&source, "native.tsx", lang);
            drop(owner);
            (source, findings, reference)
        }
    };
    let actual: Vec<_> = actual.iter().map(native).collect();
    let expected: Vec<_> = reference.diagnostics.iter().map(|d| json!({
        "rule":d.rule_name,"severity":d.severity,"message":d.message.as_str(),"start":d.start,"end":d.end,"help":d.help.as_ref().map(|s|s.as_str()),
        "labels":d.labels.iter().map(|l|json!({"message":l.message.as_str(),"start":l.start,"end":l.end})).collect::<Vec<_>>(),"fix":d.fix,
    })).collect();
    assert_eq!(reference.error_count, 0);
    assert_eq!(reference.warning_count, actual.len());
    assert_eq!(actual, expected, "{source}");
    for row in &actual {
        let start = row["start"].as_u64().unwrap() as usize;
        let end = row["end"].as_u64().unwrap() as usize;
        let text = source.get(start..end).unwrap();
        assert!(text.starts_with("tabindex") || text.starts_with("tabIndex"));
        assert!(text.ends_with('"') || text.ends_with('\''));
    }
    actual
}
