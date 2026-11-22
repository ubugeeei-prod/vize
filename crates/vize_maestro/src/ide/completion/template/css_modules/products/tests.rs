#![expect(
    clippy::unwrap_used,
    reason = "authored fixtures and complete product outcomes are asserted"
)]
use super::*;
use crate::ide::{CompletionService, IdeContext};
use crate::server::ServerState;
use std::collections::BTreeSet;
use tower_lsp::lsp_types::{CompletionResponse, Url};
use vize_canon::virtual_ts::TemplateGlobal;

const CURSOR: &str = "$style.";
const TEMPLATE: &str = "<template><p>{{ $style.card }}</p></template>\r\n";
const STYLES: &[(&str, &str)] = &[
    (
        "css",
        ".card, .accent { color: red } :global(.hidden) { color: blue }",
    ),
    ("scss", ".card { &-body { color: red } }"),
    ("less", ".card { &-body { color: red } }"),
    ("sass", ".card\n  &-body\n    color: red\n"),
    ("stylus", ".card\n  &-body\n    color red\n"),
    ("styl", ".card\n  &-body\n    color red\n"),
];

fn options() -> VirtualTsOptions {
    VirtualTsOptions {
        template_globals: vec![
            TemplateGlobal {
                name: "$other".into(),
                type_annotation: "number".into(),
                default_value: "1".into(),
            },
            TemplateGlobal {
                name: "$style".into(),
                type_annotation: "never".into(),
                default_value: "null".into(),
            },
        ],
        css_modules: vec!["$base".into()],
        auto_import_stubs: vec!["declare function useProject(): string;".into()],
        auto_import_bindings: vec!["explicit".into()],
        external_template_bindings: vec!["GlobalBadge".into()],
        reference_paths: vec!["../global.d.ts".into()],
        strict_instance_globals: true,
    }
}

fn original(
    ctx: &IdeContext<'_>,
    base: &VirtualTsOptions,
    strict: bool,
    imports: bool,
) -> StyleModuleToolProducts {
    let checker = base.for_style_modules(ctx.descriptor().unwrap(), strict, imports);
    let completions = super::super::css_module_class_completions(ctx);
    StyleModuleToolProducts {
        checker,
        completions,
    }
}

fn assert_equivalent(a: &StyleModuleToolProducts, b: &StyleModuleToolProducts, name: &str) {
    let (a_opts, b_opts) = (&a.checker, &b.checker);
    let globals = |o: &VirtualTsOptions| {
        o.template_globals
            .iter()
            .map(|g| {
                (
                    g.name.clone(),
                    g.type_annotation.clone(),
                    g.default_value.clone(),
                )
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(globals(a_opts), globals(b_opts), "{name}");
    assert_eq!(a_opts.css_modules, b_opts.css_modules, "{name}");
    assert_eq!(a_opts.auto_import_stubs, b_opts.auto_import_stubs, "{name}");
    assert_eq!(
        a_opts.auto_import_bindings, b_opts.auto_import_bindings,
        "{name}"
    );
    assert_eq!(
        a_opts.external_template_bindings, b_opts.external_template_bindings,
        "{name}"
    );
    assert_eq!(a_opts.reference_paths, b_opts.reference_paths, "{name}");
    assert_eq!(
        a_opts.strict_instance_globals, b_opts.strict_instance_globals,
        "{name}"
    );
    assert_eq!(a.completions, b.completions, "{name}");
}

fn compare(
    ctx: &IdeContext<'_>,
    base: &VirtualTsOptions,
    strict: bool,
    imports: bool,
    name: &str,
) -> StyleModuleToolProducts {
    let product = ctx
        .style_module_products(base, strict, imports)
        .unwrap()
        .unwrap();
    assert_equivalent(&product, &original(ctx, base, strict, imports), name);
    product
}

fn in_document<T>(source: &str, at: usize, f: impl FnOnce(&IdeContext<'_>) -> T) -> T {
    let state = ServerState::new();
    let uri = Url::parse("file:///project/Card.vue").unwrap();
    state
        .documents
        .open(uri.clone(), source.to_owned(), 1, "vue".into());
    let ctx = IdeContext::new(&state, &uri, at).unwrap();
    f(&ctx)
}

#[test]
fn all_style_dialects_share_complete_checker_options_and_published_member_responses() {
    let base = options();
    for (language, body) in STYLES {
        let source = format!("{TEMPLATE}<style module lang='{language}'>{body}</style>");
        in_document(
            &source,
            source.find(CURSOR).unwrap() + CURSOR.len(),
            |ctx| {
                for strict in [false, true] {
                    for imports in [false, true] {
                        let result = compare(ctx, &base, strict, imports, language);
                        let published = match CompletionService::complete(ctx).unwrap() {
                            CompletionResponse::Array(items) => items,
                            CompletionResponse::List(list) => list.items,
                        };
                        assert_eq!(result.completions.as_ref(), Some(&published));
                        let (labels, object): (&[&str], &str) = match *language {
                            "css" => (
                                &["accent", "card"],
                                r#"{ "accent": string; "card": string; }"#,
                            ),
                            _ => (
                                &["card", "card-body"],
                                r#"{ "card": string; "card-body": string; }"#,
                            ),
                        };
                        assert_eq!(
                            published
                                .iter()
                                .map(|item| item.label.as_str())
                                .collect::<Vec<_>>(),
                            labels,
                            "{language}"
                        );
                        let annotation = if strict {
                            object.to_owned()
                        } else {
                            format!("Record<string, string> & {object}")
                        };
                        assert_eq!(
                            result
                                .checker
                                .template_globals
                                .iter()
                                .find(|g| g.name == "$style")
                                .map(|g| g.type_annotation.as_str()),
                            Some(annotation.as_str()),
                            "{language}"
                        );
                    }
                }
            },
        );
    }
}

#[test]
fn duplicate_external_dynamic_and_unresolved_modules_keep_each_products_policy() {
    let base = options();
    let sources = [
        "<style module>.a{color:red}</style><style module>.b{color:blue}</style>",
        "<style module src='./theme.css'></style><style module>.local{}</style>",
        "<style module>@import './external.css'; .local { color: red }</style>",
        "<style module>.local { composes: card from './card.css' }</style>",
        "<style module lang='scss'>.known{} .#{$dynamic}{color:red}</style>",
        "<style module lang='less'>.local{color:red</style>",
        "<style module lang='unknown'>.unreadable{}</style>",
        "<style scoped>.plain{}</style>",
    ];
    for styles in sources {
        let source = format!("{TEMPLATE}{styles}");
        in_document(
            &source,
            source.find(CURSOR).unwrap() + CURSOR.len(),
            |ctx| {
                for imports in [false, true] {
                    compare(ctx, &base, true, imports, styles);
                }
            },
        );
    }
}

#[test]
fn stale_buffers_and_mismatched_owned_style_ranges_do_not_publish() {
    let source = format!("{TEMPLATE}<style module>.a{{}}</style>");
    let state = ServerState::new();
    let uri = Url::parse("file:///project/Stale.vue").unwrap();
    state
        .documents
        .open(uri.clone(), source.clone(), 1, "vue".to_owned());
    let ctx = IdeContext::new(&state, &uri, 0).unwrap();
    let mut descriptor = (**ctx.descriptor().unwrap()).clone();
    let style = descriptor.styles.first_mut().unwrap();
    style.content = ".z{}".into();
    assert!(matches!(
        options().observe_style_modules(&descriptor, true, false, |_, _, _| {}),
        Err(SourceFrameError::BlockNotRootSlice)
    ));
    state.documents.close(&uri);
    assert!(matches!(
        ctx.style_module_products(&options(), true, false),
        Err(StyleModuleRefusal::Closed)
    ));
    state
        .documents
        .open(uri.clone(), source, 1, "vue".to_owned());
    assert!(matches!(
        ctx.style_module_products(&options(), true, false),
        Err(StyleModuleRefusal::Superseded)
    ));
}

fn original_modules(mut read: impl FnMut(&str, &str, Vec<usize>)) -> (usize, usize, usize) {
    let list = std::fs::read(std::env::var_os("VIZE_STYLE_MODULE_INPUTS").unwrap()).unwrap();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let (mut files, mut invalid, mut parsed) = (0, 0, 0);
    for path in list.split(|&b| b == 0).filter(|p| !p.is_empty()) {
        let filename = std::str::from_utf8(path).unwrap();
        let bytes = std::fs::read(root.join(filename)).unwrap();
        files += 1;
        let Ok(source) = std::str::from_utf8(&bytes) else {
            invalid += 1;
            continue;
        };
        let Some(descriptor) = vize_incremental::parse_borrowed_descriptor(filename, source) else {
            continue;
        };
        parsed += 1;
        if !descriptor.styles.iter().any(|style| style.module.is_some()) {
            continue;
        }
        let mut offsets = BTreeSet::from([0]);
        if let Some(template) = &descriptor.template {
            for name in descriptor
                .styles
                .iter()
                .filter_map(|style| style.module.as_deref())
            {
                let search = format!("{name}.");
                offsets.extend(
                    template
                        .content
                        .match_indices(&search)
                        .map(|(i, _)| template.loc.start + i + search.len()),
                );
            }
        }
        read(filename, source, offsets.into_iter().collect());
    }
    (files, invalid, parsed)
}

#[test]
#[ignore = "original module structures, all checker options, and completion-item wire payloads"]
fn original_style_module_products_match() {
    let base = options();
    let (mut module_files, mut checks, mut items) = (0, 0, 0);
    let (files, invalid, parsed) = original_modules(|name, source, offsets| {
        module_files += 1;
        for at in offsets {
            in_document(source, at, |ctx| {
                for strict in [false, true] {
                    for imports in [false, true] {
                        let result = compare(ctx, &base, strict, imports, name);
                        checks += 1;
                        items += result.completions.as_ref().map_or(0, Vec::len);
                    }
                }
            });
        }
    });
    assert!(module_files > 0 && checks > 0);
    eprintln!(
        "style_modules files={files} invalid={invalid} parsed={parsed} modules={module_files} checks={checks} items={items}"
    );
}

#[cfg(feature = "micro-profiling")]
mod profile;
