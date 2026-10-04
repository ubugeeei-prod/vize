//! #7831: exact reporter templates, emitted children, and real Vue rendering.
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "regressions retain complete compiler and runtime observations"
)]

use serde_json::{Value, json};
use std::{
    io::Write,
    path::Path,
    process::{Command, Stdio},
};
use vize_atelier_core::{
    CodegenMode, CompilerError, ErrorCode, TemplateChildNode, WhitespaceStrategy,
    parser::with_whitespace_strategy,
};
use vize_atelier_dom::{DomCompilerOptions, compile_template_with_options};
use vize_atelier_ssr::{SsrCompilerOptions, compile_ssr_with_options};
use vize_l0::Allocator;

const FIXTURES: &str = include_str!(
    "../../../tests/_fixtures/differential/compiler/conditional-branch-whitespace/runtime.expected.json"
);
const NUXT_ROOT: &str = include_str!(
    "../../../tests/_fixtures/differential/compiler/conditional-branch-whitespace/nuxt-root.template.txt"
);
const MINIMAL: &str = include_str!(
    "../../../tests/_fixtures/differential/compiler/conditional-branch-whitespace/minimal-suspense.template.txt"
);
const INLINE: &str = include_str!(
    "../../../tests/_fixtures/differential/compiler/conditional-branch-whitespace/inline.template.txt"
);

fn dom_options() -> DomCompilerOptions {
    DomCompilerOptions {
        mode: CodegenMode::Module,
        prefix_identifiers: true,
        comments: true,
        ..Default::default()
    }
}

fn assert_diagnostics(template: &str, errors: &[CompilerError]) {
    // Keep the reporter's original <div />. The ordinary compiler recovers it
    // with this existing diagnostic, also seen in the real Nuxt build.
    let self_closing = if template == NUXT_ROOT {
        Some("<div v-if=\"abortRender\" />")
    } else if template == MINIMAL {
        Some("<div v-if=\"a\" />")
    } else {
        None
    };
    assert_eq!(
        errors.len(),
        usize::from(self_closing.is_some()),
        "{errors:?}"
    );
    if let Some(source) = self_closing {
        let error = errors.first().expect("unchanged recoverable diagnostic");
        assert_eq!(error.code, ErrorCode::ExtendPoint);
        assert_eq!(
            error.message,
            "Invalid self-closing syntax on non-void HTML element was rewritten as an empty element with an explicit end tag."
        );
        assert_eq!(
            error
                .loc
                .as_ref()
                .expect("diagnostic source span")
                .span
                .slice(template),
            source
        );
    }
}

#[test]
fn successful_chains_remove_only_their_whitespace_gaps() {
    for whitespace in [WhitespaceStrategy::Preserve, WhitespaceStrategy::Condense] {
        let allocator = Allocator::new();
        let (root, errors, output) = with_whitespace_strategy(whitespace, || {
            compile_template_with_options(&allocator, NUXT_ROOT, dom_options())
        });
        assert_diagnostics(NUXT_ROOT, &errors);
        let Some(TemplateChildNode::Element(suspense)) = root.children.first() else {
            panic!("the reporter Suspense root must survive compilation");
        };
        assert_eq!(suspense.children.len(), 1, "{}", output.code);
        let Some(TemplateChildNode::If(chain)) = suspense.children.first() else {
            panic!("Suspense must receive one conditional child");
        };
        assert_eq!(chain.branches.len(), 5);

        let allocator = Allocator::new();
        let (root, errors, _) = with_whitespace_strategy(whitespace, || {
            compile_template_with_options(
                &allocator,
                "<p><i v-if=\"a\">1</i> <!-- between --> <b v-else>2</b> <em>{{ end }}</em></p>",
                dom_options(),
            )
        });
        assert!(errors.is_empty(), "{errors:?}");
        let Some(TemplateChildNode::Element(parent)) = root.children.first() else {
            panic!("parent element must survive compilation");
        };
        assert_eq!(
            parent.children.len(),
            4,
            "comments and outside space remain"
        );
        assert!(matches!(
            parent.children.first(),
            Some(TemplateChildNode::If(_))
        ));
        assert!(parent.children.iter().any(|child| matches!(child,
            TemplateChildNode::Comment(comment) if comment.content == " between ")));
        assert!(parent.children.iter().any(|child| matches!(child,
            TemplateChildNode::Text(text) if text.content == " ")));
        assert!(
            matches!(parent.children.last(), Some(TemplateChildNode::Element(el)) if el.tag == "em")
        );
    }
}

#[test]
fn non_whitespace_between_branches_still_reports_non_adjacency() {
    let allocator = Allocator::new();
    let (_, errors, _) = with_whitespace_strategy(WhitespaceStrategy::Preserve, || {
        compile_template_with_options(
            &allocator,
            "<p><i v-if=\"a\">1</i> x <b v-else>2</b></p>",
            dom_options(),
        )
    });
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert_eq!(
        errors.first().expect("adjacency error").code,
        ErrorCode::VElseNoAdjacentIf
    );
}

#[test]
fn reporter_and_controls_mount_and_ssr_like_vue_in_production() {
    let fixtures: Vec<Value> = serde_json::from_str(FIXTURES).expect("pinned runtime corpus");
    let mut cases = Vec::new();
    for (mode, whitespace) in [
        ("preserve", WhitespaceStrategy::Preserve),
        ("condense", WhitespaceStrategy::Condense),
    ] {
        for fixture in &fixtures {
            let template = match fixture["templateFile"].as_str() {
                Some("nuxt-root.template.txt") => NUXT_ROOT,
                Some("minimal-suspense.template.txt") => MINIMAL,
                Some("inline.template.txt") => INLINE,
                None => fixture["template"].as_str().expect("control template"),
                Some(other) => panic!("unknown corpus template {other}"),
            };
            let allocator = Allocator::new();
            let (_, errors, dom) = with_whitespace_strategy(whitespace, || {
                compile_template_with_options(&allocator, template, dom_options())
            });
            assert_diagnostics(template, &errors);
            let allocator = Allocator::new();
            let (_, errors, ssr) = with_whitespace_strategy(whitespace, || {
                compile_ssr_with_options(&allocator, template, SsrCompilerOptions::default())
            });
            assert_diagnostics(template, &errors);
            cases.push(json!({
                "name": fixture["name"], "whitespace": mode, "template": template,
                "states": fixture["states"],
                "dom": format!("{}\n{}", dom.preamble, dom.code),
                "ssr": format!("{}\n{}", ssr.preamble, ssr.code),
            }));
        }
    }
    let mut child = Command::new("node")
        .arg(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../tests/tooling/support/conditional-branch-whitespace.mjs"),
        )
        .env("NODE_ENV", "production")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("run the real production Vue oracle");
    child
        .stdin
        .take()
        .expect("oracle stdin")
        .write_all(
            serde_json::to_string(&cases)
                .expect("serialize compiler observations")
                .as_bytes(),
        )
        .expect("send complete DOM and SSR modules");
    let output = child.wait_with_output().expect("Vue oracle exits");
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).lines().count(),
        36,
        "all 18 states run in both whitespace modes"
    );
}
