use std::{
    borrow::Cow,
    cell::RefCell,
    fs,
    io::{BufWriter, Write},
    path::Path,
};
use vize_carton::i18n::{Locale, translator};
use vize_l0::{
    Allocator, SourceRoot, String,
    config::{VueDialect, VueVersion},
    diag::{Diagnostic, MessageLookup},
};
use vize_l1::{container::vue::DescriptorOptions, markup::SurfaceParseOptions};
use vize_l1_to_l2::native_file::lower_sfc_native;
use vize_l2::lang::js::JsxFileProducer;
use vize_patina::native::{
    NativeLintFinding,
    canonical::{NativeCanonicalLint, NativeCanonicalLintRefusal},
};

mod group;
mod profile;
mod regions;

type Answer = Result<Vec<NativeLintFinding>, NativeCanonicalLintRefusal>;
type Findings = Result<Vec<(&'static str, Diagnostic)>, NativeCanonicalLintRefusal>;

fn options() -> DescriptorOptions {
    DescriptorOptions {
        version: VueVersion::V3,
        dialect: VueDialect::Vue,
        template: SurfaceParseOptions::default(),
    }
}

pub(crate) fn separate(
    lint: &NativeCanonicalLint<'_, '_>,
    messages: &impl MessageLookup,
) -> (Answer, Answer) {
    (lint.tabindex_no_positive(messages), lint.img_alt(messages))
}

fn complete(answer: Answer) -> Findings {
    answer.map(|rows| {
        rows.into_iter()
            .map(|r| (r.rule_name(), r.into_diagnostic()))
            .collect()
    })
}

// Each message carries its call index. Comparing both packets and the key log
// detects eager localization, missing lookups and reordered calls on refusals.
#[derive(Default)]
struct OrderedCatalog(RefCell<Vec<String>>);
impl MessageLookup for OrderedCatalog {
    fn lookup(&self, key: &str) -> Cow<'static, str> {
        let mut keys = self.0.borrow_mut();
        let result = vize_l0::cstr!("{}:{key}", keys.len()).into();
        keys.push(key.into());
        Cow::Owned(result)
    }
}

fn compare(
    lint: &NativeCanonicalLint<'_, '_>,
    catalog: &impl MessageLookup,
) -> (Findings, Findings) {
    let old_tab = complete(lint.tabindex_no_positive(catalog));
    let old_img = complete(lint.img_alt(catalog));
    let (tab, img) = group::requested(lint, catalog);
    let tab = complete(tab);
    let img = complete(img);
    assert_eq!(old_tab, tab);
    assert_eq!(old_img, img);
    (tab, img)
}

fn ordered(lint: &NativeCanonicalLint<'_, '_>) {
    let a = OrderedCatalog::default();
    let tab = complete(lint.tabindex_no_positive(&a));
    let img = complete(lint.img_alt(&a));
    let b = OrderedCatalog::default();
    let (new_tab, new_img) = group::requested(lint, &b);
    assert_eq!(tab, complete(new_tab));
    assert_eq!(img, complete(new_img));
    assert_eq!(a.0, b.0);
}

#[derive(Default, Debug)]
struct Counts {
    files: usize,
    invalid_utf8: usize,
    lowering_refusals: usize,
    owner_refusals: usize,
    admitted: usize,
    tab_findings: usize,
    img_findings: usize,
    tab_refusals: usize,
    img_refusals: usize,
    capture: Option<BufWriter<fs::File>>,
}

fn check(lint: &NativeCanonicalLint<'_, '_>, count: &mut Counts) -> std::io::Result<()> {
    count.admitted += 1;
    ordered(lint);
    for &locale in Locale::ALL {
        let (tab, img) = compare(lint, &translator().for_locale(locale));
        if let Some(log) = count.capture.as_mut() {
            writeln!(log, "{locale:?} {tab:?} {img:?}")?;
        }
        if locale == Locale::En {
            match tab {
                Ok(rows) => count.tab_findings += rows.len(),
                Err(_) => count.tab_refusals += 1,
            }
            match img {
                Ok(rows) => count.img_findings += rows.len(),
                Err(_) => count.img_refusals += 1,
            }
        }
    }
    Ok(())
}

fn original(text: &str, path: &str, count: &mut Counts) -> std::io::Result<()> {
    if let Some(log) = count.capture.as_mut() {
        writeln!(log, "source {path:?}")?;
    }
    with_original(text, path, count, check)
}

fn with_original(
    text: &str,
    path: &str,
    count: &mut Counts,
    observe: impl FnOnce(&NativeCanonicalLint<'_, '_>, &mut Counts) -> std::io::Result<()>,
) -> std::io::Result<()> {
    let arena = Allocator::default();
    let syntax_path = path.strip_suffix(".txt").unwrap_or(path);
    if syntax_path.ends_with(".tsx") || syntax_path.ends_with(".jsx") {
        let ty = if syntax_path.ends_with(".tsx") {
            oxc_span::SourceType::tsx()
        } else {
            oxc_span::SourceType::jsx()
        }
        .with_module(true);
        let parser = oxc_parser::Parser::new(&arena, text, ty).parse_observed();
        let Ok(root) = SourceRoot::new(text) else {
            count.lowering_refusals += 1;
            return Ok(());
        };
        let block = root.whole_block();
        let Ok(mut producer) = JsxFileProducer::new(&arena, parser, block, 0) else {
            count.lowering_refusals += 1;
            return Ok(());
        };
        if producer.walk().is_err() {
            count.lowering_refusals += 1;
            return Ok(());
        }
        let Ok(owner) = producer.finish_markup() else {
            count.lowering_refusals += 1;
            return Ok(());
        };
        let Ok(lint) = NativeCanonicalLint::from_jsx(&owner) else {
            count.owner_refusals += 1;
            return Ok(());
        };
        observe(&lint, count)
    } else {
        let observation = lower_sfc_native(&arena, text, options());
        let Some(owner) = observation.admitted() else {
            count.lowering_refusals += 1;
            return Ok(());
        };
        observe(&NativeCanonicalLint::from_vue(owner), count)
    }
}

#[test]
fn authentic_owners_keep_each_rule_and_ordered_catalog_effects() {
    let mut count = Counts::default();
    for body in [
        "<section><img/><p tabindex='1'/><img alt=''/><p tabindex='-2'/></section>",
        "<section><p tabindex='2'/><img/><img/></section>",
        "<section><img/><i/><p tabindex='3'/><!-- vize-disable --><img/></section>",
        "<div tabindex='2'><img/><Widget/></div>",
        "<section><img/><p/><img v-bind='attrs'/><p tabindex='2'/></section>",
        "<section><img alt=''/><i tabindex='0'/></section>",
        "<img/>",
        "plain text",
        "<img tabindex='2'/>",
        "<Widget tabindex='2'/>",
        "<!-- vize-disable -->",
        "<img :alt='label'/>",
    ] {
        let text = vize_l0::cstr!("<template>{body}</template>");
        original(&text, "fixture.vue", &mut count).unwrap();
    }
    for path in ["fixture.jsx", "fixture.tsx"] {
        original(
            "const view = <section><img/><p tabIndex='2'/><img alt=''/></section>",
            path,
            &mut count,
        )
        .unwrap();
    }
    assert!(count.admitted >= 6, "{count:?}");
    assert!(
        count.tab_findings > 0
            && count.img_findings > 0
            && count.img_refusals > 0
            && count.tab_refusals > 0,
        "{count:?}"
    );
}

#[test]
fn script_only_file_has_two_empty_results_and_no_catalog_effects() {
    let arena = Allocator::default();
    let source = "<script setup>const value = 1</script>";
    let lowering = lower_sfc_native(&arena, source, options());
    let lint = NativeCanonicalLint::from_vue(lowering.admitted().unwrap());
    assert_eq!(lint.l2ir().artifact().node_count(), 0);
    let catalog = OrderedCatalog::default();
    let (tab, image) = group::requested(&lint, &catalog);
    assert!(tab.unwrap().is_empty() && image.unwrap().is_empty());
    assert!(catalog.0.borrow().is_empty());
}

#[test]
fn image_refusal_preserves_the_other_rules_findings() {
    let mut count = Counts::default();
    original(
        "<template><div tabindex='2'><svg><img/></svg></div></template>",
        "fixture.vue",
        &mut count,
    )
    .unwrap();
    assert_eq!(
        (count.admitted, count.tab_findings, count.img_refusals),
        (1, 1, 1)
    );
    assert_eq!(count.tab_refusals, 0);
}

#[test]
#[ignore = "requires a NUL-delimited manifest of original Vue, JSX or TSX files"]
fn original_sources() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let list = fs::read(std::env::var_os("VIZE_CANONICAL_LINT_INPUTS").unwrap()).unwrap();
    let mut count = Counts {
        capture: std::env::var_os("VIZE_CANONICAL_LINT_CAPTURE")
            .map(|path| BufWriter::new(fs::File::create(path).unwrap())),
        ..Counts::default()
    };
    for raw in list.split(|&b| b == 0).filter(|s| !s.is_empty()) {
        let path = core::str::from_utf8(raw).unwrap();
        let file = fs::read(root.join(path)).unwrap();
        count.files += 1;
        if let Ok(text) = core::str::from_utf8(&file) {
            original(text, path, &mut count).unwrap();
        } else {
            count.invalid_utf8 += 1;
        }
    }
    assert_eq!(
        count.files,
        count.invalid_utf8 + count.lowering_refusals + count.owner_refusals + count.admitted
    );
    assert!(count.admitted > 0);
    if let Some(mut log) = count.capture.take() {
        log.flush().unwrap();
    }
    eprintln!("{count:?}");
}
