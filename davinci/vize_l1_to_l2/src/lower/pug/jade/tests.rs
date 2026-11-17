use alloc::borrow::ToOwned;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use vize_l0::diag::{PartKind, Severity};
use vize_l1::markup::pug::PugProfile;

use crate::lower::pug::{PugRendering, PugTemplate, derive_template_source_profiled};

fn derive(source: &str, profile: PugProfile) -> PugTemplate {
    derive_template_source_profiled(source, PugRendering::Pug, profile)
}

/// `(severity, authored text at the span, suggestion)` of every diagnostic.
fn findings(source: &str, template: &PugTemplate) -> Vec<(Severity, String, Option<String>)> {
    template
        .diagnostics
        .iter()
        .map(|diagnostic| {
            let at = source
                .get(diagnostic.span.start as usize..diagnostic.span.end as usize)
                .unwrap_or_default()
                .to_owned();
            let fix = diagnostic
                .parts
                .iter()
                .find(|part| part.kind == PartKind::Suggestion)
                .map(|part| part.message.to_string());
            (diagnostic.severity(), at, fix)
        })
        .collect()
}

#[test]
fn dashed_iteration_is_jade_iteration_with_a_fix() {
    let source = "ul\n  - each item in [1, 2]\n    li= item\n  - for n in [3]\n    li= n\n";
    let jade = derive(source, PugProfile::Jade1);
    assert_eq!(
        jade.html.as_str(),
        "<ul><li>1</li><li>2</li><li>3</li></ul>"
    );
    assert_eq!(
        findings(source, &jade),
        [
            (Severity::Warning, "- ".to_owned(), Some(String::new())),
            (Severity::Warning, "- ".to_owned(), Some(String::new())),
        ]
    );
    // pug 3 still refuses the dash.
    assert!(derive(source, PugProfile::Pug3).has_errors());
}

#[test]
fn a_bodiless_mixin_is_jade_s_legacy_call() {
    let source = "mixin foo(x)\n  p= x\nmixin foo('a')\n";
    let jade = derive(source, PugProfile::Jade1);
    assert_eq!(jade.html.as_str(), "<p>a</p>");
    assert_eq!(
        findings(source, &jade),
        [(
            Severity::Warning,
            "mixin foo('a')".to_owned(),
            Some("+foo('a')".to_owned())
        )]
    );
    assert!(derive(source, PugProfile::Pug3).has_errors());
}

#[test]
fn each_of_iterates_and_warns_that_jade_renders_a_literal_element() {
    let source = "ul\n  each item of [1, 2]\n    li= item\n";
    let jade = derive(source, PugProfile::Jade1);
    let pug = derive(source, PugProfile::Pug3);
    assert_eq!(jade.html, pug.html);
    assert_eq!(
        findings(source, &jade),
        [(Severity::Warning, "each item of [1, 2]".to_owned(), None)]
    );
    assert_eq!(findings(source, &pug), []);
}

#[test]
fn attribute_interpolation_is_kept_and_warned_with_a_binding_fix() {
    let source = "a(href=\"/users/#{id}\") x\n";
    let jade = derive(source, PugProfile::Jade1);
    let pug = derive(source, PugProfile::Pug3);
    assert_eq!(jade.html, pug.html);
    assert_eq!(jade.html.as_str(), "<a href=\"/users/#{id}\">x</a>");
    assert_eq!(
        findings(source, &jade),
        [(
            Severity::Warning,
            "#{id}".to_owned(),
            Some(":href=\"`/users/${id}`\"".to_owned())
        )]
    );
    assert_eq!(findings(source, &pug), []);
    // An escaped `\#{` is text in jade too; a value a template literal
    // cannot carry keeps the warning without a fix.
    let escaped = "a(href=\"\\#{id}\") x\n";
    assert_eq!(findings(escaped, &derive(escaped, PugProfile::Jade1)), []);
    let quoted = "a(title='say \"#{name}\"') x\n";
    assert_eq!(
        findings(quoted, &derive(quoted, PugProfile::Jade1)),
        [(Severity::Warning, "#{name}".to_owned(), None)]
    );
}

#[test]
fn the_pug3_profile_derives_jade_only_syntax_byte_identically() {
    for source in [
        "ul\n  each item of [1, 2]\n    li= item\n",
        "a(href=\"/users/#{id}\") x\n",
        "div(v-if=\"ok\" :x=\"y\")\n",
        "p #{'lit'} {{ vue }}\n",
    ] {
        let default = crate::lower::pug::derive_template_source(source);
        let pug = derive(source, PugProfile::Pug3);
        assert_eq!(pug.html, default.html, "{source}");
        assert_eq!(pug.diagnostics, default.diagnostics, "{source}");
    }
}
