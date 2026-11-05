use super::HtmlEndTags;
use crate::linter::Linter;
use crate::rule::RuleRegistry;

fn linter() -> Linter {
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(HtmlEndTags));
    Linter::with_registry(registry)
}

fn warnings(source: &str) -> Vec<String> {
    linter()
        .lint_template(source, "test.vue")
        .diagnostics
        .into_iter()
        .filter(|diagnostic| diagnostic.rule_name == "vue/html-end-tags")
        .map(|diagnostic| diagnostic.message.to_string())
        .collect()
}

#[test]
fn accepts_closed_optional_and_ordinary_tags() {
    for source in [
        "<ul><li>a</li><li>b</li></ul>",
        "<p>hello</p>",
        "<p>hello</P>",
        "<br><img src=\"x\">",
        "<div></div>",
        "<div />",
        "<Comp />",
        "<Comp></Comp>",
        "<ul><li><Link href=\"/account\">Overview</Link></li></ul>",
        "<p><Input>content</Input></p>",
        "<table><tr><td>a</td></tr></table>",
        "<ul><li>a<ul><li>b</li></ul></li></ul>",
        "<li v-for=\"x in xs\">{{ x }}</li>",
    ] {
        assert_eq!(warnings(source), Vec::<String>::new(), "{source}");
    }
}

#[test]
fn reports_omitted_optional_end_tags() {
    assert_eq!(warnings("<ul><li>a</ul>"), ["'<li>' should have end tag."]);
    assert_eq!(warnings("<p>hello"), ["'<p>' should have end tag."]);
    assert_eq!(
        warnings("<select><option>A<option>B</select>"),
        [
            "'<option>' should have end tag.",
            "'<option>' should have end tag."
        ]
    );
    assert_eq!(warnings("<LI>item"), ["'<LI>' should have end tag."]);
    assert_eq!(
        warnings("<div><p>hello</div>"),
        ["'<p>' should have end tag."]
    );
    assert_eq!(
        warnings("<table><tr><td>a</tr></table>"),
        ["'<td>' should have end tag."]
    );
    assert_eq!(
        warnings("<table><td>a</table>"),
        ["'<td>' should have end tag."]
    );
    assert_eq!(
        warnings("<table><tr><td>a</td></table>"),
        ["'<tr>' should have end tag."]
    );
    assert_eq!(
        warnings("<table><tbody><tr><td>a</td></tr></table>"),
        ["'<tbody>' should have end tag."]
    );
    assert_eq!(
        warnings("<p>a<p>b"),
        ["'<p>' should have end tag.", "'<p>' should have end tag."]
    );
}

#[test]
fn leaves_parser_owned_missing_end_tags_alone() {
    for source in ["<div>", "<div><span></div>", "<ul>"] {
        let result = linter().lint_template(source, "test.vue");
        assert!(
            result
                .diagnostics
                .iter()
                .all(|diagnostic| diagnostic.rule_name != "vue/html-end-tags"),
            "{source}: {:?}",
            result.diagnostics
        );
    }
}

#[test]
fn fix_inserts_the_end_tag_after_the_content() {
    let source = "<ul><li>a</ul>";
    let result = linter().lint_template(source, "test.vue");
    let fix = result.diagnostics[0].fix.as_ref().expect("fix");
    assert_eq!(fix.apply(source).as_str(), "<ul><li>a</li></ul>");

    let source = "<table><tr><td>a</tr></table>";
    let result = linter().lint_template(source, "test.vue");
    let fix = result.diagnostics[0].fix.as_ref().expect("fix");
    assert_eq!(
        fix.apply(source).as_str(),
        "<table><tr><td>a</td></tr></table>"
    );
}

#[test]
fn reports_inside_an_sfc_template() {
    let source = "<template><ul><li>a</ul></template>";
    let result = linter().lint_sfc(source, "test.vue");
    let found: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.rule_name == "vue/html-end-tags")
        .map(|diagnostic| diagnostic.message.as_str())
        .collect();
    assert_eq!(found, ["'<li>' should have end tag."]);
}
