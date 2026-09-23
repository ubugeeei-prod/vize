//! Temporary candidate scan: Relief visitor diagnostics vs the S2 facade.
//! Deleted once a second SFC facade rule is chosen.

use super::{TEMPLATES, rule_fixtures};
use crate::context::LintContext;
use crate::diagnostic::LintDiagnostic;
use crate::ir::TemplateSyntax;
use crate::markup::{MarkupContext, MarkupDocument, S2Template};
use crate::rule::RuleRegistry;
use crate::visitor::LintVisitor;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use vize_armature::Parser;
use vize_atelier_sfc::{SfcParseOptions, parse_sfc};
use vize_s0::Allocator;

const LOG: &str = "/tmp/facade-probe.txt";

const EXTRA: &[&str] = &[
    r#"<marquee>x</marquee>"#,
    r#"<blink>x</blink>"#,
    r#"<center>x</center>"#,
    r#"<font color="red">x</font>"#,
    r#"<big>x</big><tt>y</tt><strike>z</strike>"#,
    r#"<div accesskey="a"></div>"#,
    r#"<div :accesskey="'a'"></div>"#,
    r#"<div :[accesskey]="'a'"></div>"#,
    r#"<input autofocus>"#,
    r#"<input :autofocus="true">"#,
    r#"<input :[autofocus]="on">"#,
    r#"<ul role="list"></ul>"#,
    r#"<nav role="navigation">x</nav>"#,
    r#"<button role="button">ok</button>"#,
    r##"<a href="#" role="link">x</a>"##,
    r#"<div role="presentation" tabindex="0"></div>"#,
    r#"<div aria-hidden="true" tabindex="0"></div>"#,
    r#"<li v-for="i in list">{{ i }}</li>"#,
    r#"<li v-for="i in list" :key="i">{{ i }}</li>"#,
    r#"<template v-for="i in list"><span>{{ i }}</span></template>"#,
    r#"<textarea>{{ msg }}</textarea>"#,
    r#"<textarea v-model="msg"></textarea>"#,
    r#"<img src="a.png">"#,
    r#"<img src="a.png" alt="a">"#,
    r#"<iframe src="/f"></iframe>"#,
    r#"<iframe src="/f" title="t"></iframe>"#,
    r#"<iframe src="/f" sandbox=""></iframe>"#,
    r#"<a href="https://x" target="_blank">x</a>"#,
    r#"<a href="https://x" target="_blank" rel="noopener noreferrer">x</a>"#,
    r#"<button>ok</button>"#,
    r#"<button type="submit">ok</button>"#,
    r#"<input>"#,
    r#"<label><input></label>"#,
    r#"<label for="a">n</label><input id="a">"#,
    r#"<h1></h1><h3>skip</h3>"#,
    r#"<h1>t</h1><h2>u</h2>"#,
    r#"<div style="color: red"></div>"#,
    r#"<div :style="{ color: 'red' }"></div>"#,
    r#"<div class="a a"></div>"#,
    r#"<div style="color: red; color: blue"></div>"#,
    r#"<br><br>"#,
    r#"<p>a<br> <br>b</p>"#,
    r#"<time>now</time>"#,
    r#"<time datetime="2020-01-01">now</time>"#,
    r#"<dl><dt>a</dt><dt>a</dt><dd>b</dd></dl>"#,
    r#"<i class="fa fa-star"></i>"#,
    r#"<div tabindex="1"></div>"#,
    r#"<div tabindex="0"></div>"#,
    r#"<div @click="x"></div>"#,
    r#"<div @click="x" @keydown="y"></div>"#,
    r#"<video src="a.mp4"></video>"#,
    r#"<video src="a.mp4"><track kind="captions" src="c.vtt"></video>"#,
    r#"<select><option value="">pick</option></select>"#,
    r#"<ul><li>a</li></ul>"#,
    r#"<table><tr><td>1</td></tr></table>"#,
    r#"<p>Hello world</p>"#,
    r#"<div v-html="raw"></div>"#,
    r#"<Comp @vue:mounted="onMounted" />"#,
    r#"<div  class="a"  id="b"></div>"#,
    r#"<MyCenter>text</MyCenter>"#,
    r#"<div class="foo  bar"></div>"#,
    r#"<input type="text" placeholder="name">"#,
    r#"<pre>  keep   spaces</pre>"#,
    r#"<div v-pre>{{ raw }}</div>"#,
];

#[test]
#[ignore = "manual Relief-vs-S2 scan; it reports candidates and then panics"]
fn probe_sfc_facade_candidates() {
    let _ = std::fs::remove_file(LOG);
    let registry = RuleRegistry::with_all();
    let names = registry.rule_names();
    let mut indexes = Vec::new();
    for (index, rule) in registry.rules().iter().enumerate() {
        if rule.as_markup_rule().is_some() {
            indexes.push(index);
        }
    }
    log(&format!(
        "markup rules: {}",
        indexes
            .iter()
            .map(|index| names[*index])
            .collect::<Vec<_>>()
            .join(", ")
    ));

    let sources = collect_sources();
    log(&format!("sources: {}", sources.len()));

    let mut failed: Vec<Option<String>> = vec![None; indexes.len()];
    let mut hits = vec![0usize; indexes.len()];
    let mut open = indexes.len();

    for (ordinal, (label, source)) in sources.iter().enumerate() {
        if open == 0 {
            break;
        }
        if ordinal % 25 == 0 {
            log(&format!("at {ordinal}/{} open={open}", sources.len()));
        }
        let allocator = Allocator::with_capacity(source.len().saturating_mul(4).max(1024));
        let (root, _) = Parser::new(&allocator, source).parse();
        let lowered = S2Template::lower(&allocator, source);
        let markup = lowered.markup();
        let markup = crate::markup::reborrow_markup(&markup);
        let document = MarkupDocument::from_s2(markup, TemplateSyntax::Vue);

        for (slot, &index) in indexes.iter().enumerate() {
            if failed[slot].is_some() {
                continue;
            }
            let name = names[index];
            let relief = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                fingerprint(
                    &relief_diagnostics(&registry, index, &allocator, source, &root),
                    name,
                )
            }));
            let facade = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                fingerprint(
                    &facade_diagnostics(&registry, index, &allocator, source, &root, &document),
                    name,
                )
            }));
            match (relief, facade) {
                (Ok(left), Ok(right)) if left == right => {
                    hits[slot] += left.matches('\n').count();
                }
                (Ok(left), Ok(right)) => {
                    failed[slot] = Some(format!(
                        "{name} DIFF {label}\nRELIEF:\n{left}FACADE:\n{right}"
                    ));
                    open -= 1;
                }
                (Err(_), _) => {
                    failed[slot] = Some(format!("{name} PANIC relief {label}"));
                    open -= 1;
                }
                (_, Err(_)) => {
                    failed[slot] = Some(format!("{name} PANIC facade {label}"));
                    open -= 1;
                }
            }
        }
    }

    let mut matched = Vec::new();
    let mut report = String::new();
    for (slot, &index) in indexes.iter().enumerate() {
        let name = names[index];
        match &failed[slot] {
            Some(reason) => {
                let head: String = reason.chars().take(500).collect();
                report.push_str(&format!(
                    "FAIL {name} hits~ diag-lines before fail tracked separately\n{head}\n---\n"
                ));
            }
            None => {
                matched.push(format!("{name} diags={}\n", hits[slot]));
                report.push_str(&format!("MATCH {name} diagnostic-lines={}\n", hits[slot]));
            }
        }
    }
    log(&report);
    panic!(
        "probe done. matches:\n{}\nfull log {LOG}",
        if matched.is_empty() {
            "(none)".to_string()
        } else {
            matched.join("")
        }
    );
}

fn collect_sources() -> Vec<(String, String)> {
    let mut sources = Vec::new();
    for (index, source) in EXTRA.iter().enumerate() {
        sources.push((format!("extra:{index}"), (*source).to_string()));
    }
    for (name, source) in TEMPLATES {
        sources.push((format!("tmpl:{name}"), (*source).to_string()));
    }
    let fixtures = rule_fixtures::scan_rule_fixtures();
    for (label, source) in fixtures.templates {
        sources.push((format!("fix:{label}"), source.to_string()));
    }
    let matrix = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/davinci-matrix");
    let mut paths: Vec<_> = std::fs::read_dir(&matrix)
        .expect("matrix")
        .map(|entry| entry.expect("entry").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "vue"))
        .collect();
    paths.sort();
    for path in paths {
        let source = std::fs::read_to_string(&path).expect("read matrix");
        let Ok(descriptor) = parse_sfc(&source, SfcParseOptions::default()) else {
            continue;
        };
        let Some(template) = descriptor.template else {
            continue;
        };
        sources.push((
            format!(
                "matrix:{}",
                path.file_name().unwrap_or_default().to_string_lossy()
            ),
            template.content.to_string(),
        ));
    }
    sources
}

fn relief_diagnostics<'a>(
    registry: &RuleRegistry,
    index: usize,
    allocator: &'a Allocator,
    source: &'a str,
    root: &vize_relief::RootNode<'a>,
) -> Vec<LintDiagnostic> {
    let mut ctx = LintContext::new(allocator, source, "probe.vue");
    let mut keep = vec![false; registry.rules().len()];
    keep[index] = true;
    let mut visitor = LintVisitor::with_rule_filter(
        &mut ctx,
        registry.rules(),
        registry.rule_names(),
        registry.has_exit_element_rules(),
        &keep,
    );
    visitor.visit_root(root);
    ctx.into_diagnostics()
}

fn facade_diagnostics<'a>(
    registry: &RuleRegistry,
    index: usize,
    allocator: &'a Allocator,
    source: &'a str,
    root: &vize_relief::RootNode<'a>,
    document: &MarkupDocument<'a>,
) -> Vec<LintDiagnostic> {
    let mut ctx = LintContext::new(allocator, source, "probe.vue");
    let keep = vec![false; registry.rules().len()];
    {
        let mut visitor = LintVisitor::with_rule_filter(
            &mut ctx,
            registry.rules(),
            registry.rule_names(),
            registry.has_exit_element_rules(),
            &keep,
        );
        visitor.visit_root(root);
    }
    if let Some(rule) = registry.rules()[index].as_markup_rule() {
        let mut markup_ctx = MarkupContext::new(&mut ctx, document);
        document.visit_with(rule, &mut markup_ctx);
    }
    ctx.into_diagnostics()
}

fn fingerprint(diagnostics: &[LintDiagnostic], rule: &str) -> String {
    let mut out = String::new();
    for diagnostic in diagnostics.iter().filter(|d| d.rule_name == rule) {
        out.push_str(&format!(
            "{:?}\t{}\t{}\t{}\t{}\t",
            diagnostic.severity,
            diagnostic.start,
            diagnostic.end,
            diagnostic.message,
            diagnostic.help.as_deref().unwrap_or("")
        ));
        for label in &diagnostic.labels {
            out.push_str(&format!(
                "L{}:{}:{};",
                label.message, label.start, label.end
            ));
        }
        if let Some(fix) = &diagnostic.fix {
            out.push_str(&format!("\t{}", fix.message));
            for edit in &fix.edits {
                out.push_str(&format!("|{}:{}:{}", edit.start, edit.end, edit.new_text));
            }
        }
        out.push('\n');
    }
    out
}

fn log(line: &str) {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(LOG)
        .expect("log");
    let _ = writeln!(file, "{line}");
}
