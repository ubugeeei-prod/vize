use serde_json::{Value, json};
use vize_carton::i18n::{Locale, translator};
use vize_l0::{Allocator, SourceRoot};
use vize_l1::container::vue::DescriptorOptions;
use vize_l1_to_l2::native_file::lower_sfc_native;
use vize_l2::lang::js::{JsxFileProducer, JsxMarkupFile};
use vize_patina::{
    Linter, RuleRegistry,
    native::{NativeLintFinding, canonical::NativeCanonicalLint},
    rules::a11y::ImgAlt,
};

pub use crate::native_syntax_img_alt::support::native;

#[derive(Clone, Copy, Debug)]
pub enum Dialect {
    Vue,
    Jsx,
    Tsx,
}

pub fn options() -> DescriptorOptions {
    DescriptorOptions {
        version: vize_l0::config::VueVersion::V3,
        dialect: vize_l0::config::VueDialect::Vue,
        template: vize_l1::markup::SurfaceParseOptions::default(),
    }
}

pub fn jsx<'a>(a: &'a Allocator, source: &'a str, typescript: bool) -> JsxMarkupFile<'a> {
    let kind = if typescript {
        oxc_span::SourceType::tsx()
    } else {
        oxc_span::SourceType::jsx()
    }
    .with_module(true);
    let original = oxc_parser::Parser::new(a, source, kind).parse_observed();
    let block = SourceRoot::new(source).unwrap().whole_block();
    let mut producer = JsxFileProducer::new(a, original, block, 0)
        .unwrap_or_else(|rejected| panic!("original Program: {:?}", rejected.error()));
    producer.walk().unwrap();
    producer
        .finish_markup()
        .unwrap_or_else(|rejected| panic!("canonical admission: {:?}", rejected.issues()))
}

pub fn query(
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
    let before = lint.file().artifact().node_count();
    let output = lint.img_alt(&counted).unwrap();
    let lookups = if output.is_empty() { 0 } else { 2 };
    assert_eq!(counted.calls.get(), lookups);
    let repeated = lint.img_alt(&counted).unwrap();
    assert_eq!(counted.calls.get(), lookups * 2);
    assert_eq!(lint.file().artifact().node_count(), before);
    assert_eq!(
        output.iter().map(native).collect::<Vec<_>>(),
        repeated.iter().map(native).collect::<Vec<_>>()
    );
    output
}

/// The reference is the registered compatibility route, never native custody.
pub fn parity(dialect: Dialect, body: &str, newline: &str, locale: Locale) -> Vec<Value> {
    let source = match dialect {
        Dialect::Vue => vize_l0::cstr!("<!--作者😀-->{newline}<template>{body}</template>"),
        Dialect::Jsx | Dialect::Tsx => {
            vize_l0::cstr!("const prefix = '作者😀';{newline}const view = {body};")
        }
    };
    source_parity(dialect, &source, locale)
}

pub fn source_parity(dialect: Dialect, source: &str, locale: Locale) -> Vec<Value> {
    let a = Allocator::default();
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(ImgAlt));
    let linter = Linter::with_registry(registry).with_locale(locale);
    let messages = translator().for_locale(locale);
    let (actual, reference) = match dialect {
        Dialect::Vue => {
            let owner = lower_sfc_native(&a, source, options());
            let admitted = owner
                .admitted()
                .unwrap_or_else(|| panic!("{source}: {:?}", owner.issues()));
            let lint = NativeCanonicalLint::from_vue(admitted);
            assert!(core::ptr::eq(lint.file(), owner.file().unwrap().file()));
            let findings = query(&lint, &messages);
            let reference = linter.lint_sfc(source, "native.vue");
            drop(owner);
            (findings, reference)
        }
        Dialect::Jsx | Dialect::Tsx => {
            let owner = jsx(&a, source, matches!(dialect, Dialect::Tsx));
            let lint = NativeCanonicalLint::from_jsx(&owner).unwrap();
            assert!(core::ptr::eq(lint.file(), owner.file()));
            let findings = query(&lint, &messages);
            let lang = if matches!(dialect, Dialect::Tsx) {
                vize_atelier_jsx::JsxLang::Tsx
            } else {
                vize_atelier_jsx::JsxLang::Jsx
            };
            let reference = linter.lint_jsx(source, "native.tsx", lang);
            drop(owner);
            (findings, reference)
        }
    };
    // The canonical borrow has ended. Findings cannot retain arena storage.
    drop(a);
    let actual: Vec<_> = actual.iter().map(native).collect();
    let expected: Vec<_> = reference.diagnostics.iter().map(|d| json!({
        "rule_name":d.rule_name,"severity":d.severity,"message":d.message.as_str(),"start":d.start,"end":d.end,"help":d.help.as_ref().map(|s|s.as_str()),
        "labels":d.labels.iter().map(|l|json!({"message":l.message.as_str(),"start":l.start,"end":l.end})).collect::<Vec<_>>(),"fix":d.fix,
    })).collect();
    assert_eq!(reference.error_count, 0);
    assert_eq!(reference.warning_count, actual.len());
    assert_eq!(actual, expected, "{dialect:?}: {source}");
    actual
}
