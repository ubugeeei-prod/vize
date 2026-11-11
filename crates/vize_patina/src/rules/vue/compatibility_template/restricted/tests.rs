use super::NoRestrictedSyntax;
use crate::linter::Linter;
use crate::rule::RuleRegistry;
use vize_l0::String;

fn reported(entries: &[(&str, Option<&str>)], template: &str) -> Vec<(String, String)> {
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(NoRestrictedSyntax::new(
        entries
            .iter()
            .map(|(selector, message)| (String::from(*selector), message.map(String::from)))
            .collect(),
    )));
    let mut diagnostics = Linter::with_registry(registry)
        .lint_template(template, "Component.vue")
        .diagnostics;
    diagnostics.sort_by_key(|diagnostic| diagnostic.start);
    diagnostics
        .iter()
        .map(|diagnostic| {
            let text = template
                .get(diagnostic.start as usize..diagnostic.end as usize)
                .unwrap_or_default();
            (String::from(text), diagnostic.message.clone())
        })
        .collect()
}

#[test]
fn issue_conventions_report_with_their_own_messages() {
    const CAST: &str = "Keep type casts in the script block.";
    const FALLBACK: &str = "Every key of the translation proxy resolves; remove the fallback.";
    const SLOT: &str = "A slot outlet carries no inline fallback.";
    let ts = ":matches([object.name='ts'], [object.property.name='ts'])";
    let fallback = |operator: &str, chain: bool| {
        let middle = if chain {
            "ChainExpression.left > MemberExpression"
        } else {
            "MemberExpression.left"
        };
        vize_l0::cstr!("LogicalExpression[operator='{operator}'] > {middle}{ts}")
    };
    let selectors = [
        (String::from("TSAsExpression"), CAST),
        (String::from("TSNonNullExpression"), CAST),
        (String::from("TSSatisfiesExpression"), CAST),
        (String::from("TSTypeAssertion"), CAST),
        (fallback("??", false), FALLBACK),
        (fallback("??", true), FALLBACK),
        (fallback("||", false), FALLBACK),
        (fallback("||", true), FALLBACK),
        (String::from("VElement[name='slot'] > VElement"), SLOT),
        (
            String::from("VElement[name='slot'] > VExpressionContainer"),
            SLOT,
        ),
        (
            String::from("VElement[name='slot'] > VText[value=/\\S/]"),
            SLOT,
        ),
    ];
    let entries = selectors
        .iter()
        .map(|(selector, message)| (selector.as_str(), Some(*message)))
        .collect::<Vec<_>>();
    let found = reported(
        &entries,
        concat!(
            "<p>{{ (value as string).length }}</p>\n",
            "<p>{{ label!.length }}</p>\n",
            "<p>{{ ts.save ?? \"Save\" }}</p>\n",
            "<p>{{ ts?.save || \"Save\" }}</p>\n",
            "<p>{{ other.save ?? \"Save\" }}</p>\n",
            "<slot name=\"title\">Untitled</slot>\n",
            "<slot><span>{{ label }}</span></slot>\n",
            "<slot>  </slot>\n",
        ),
    );
    let expected = [
        ("value as string", CAST),
        ("label!", CAST),
        ("ts.save", FALLBACK),
        ("ts?.save", FALLBACK),
        ("Untitled", SLOT),
        ("<span>{{ label }}</span>", SLOT),
    ];
    assert_eq!(
        found,
        expected
            .iter()
            .map(|(text, message)| (String::from(*text), String::from(*message)))
            .collect::<Vec<_>>()
    );
}

#[test]
fn selectors_cover_combinators_pseudos_and_template_shapes() {
    let template = concat!(
        "<ul><li v-for=\"(item, index) in items\" :key=\"index\">{{ item }}</li></ul>\n",
        "<button @click=\"run(); done = true\">x</button><button @click=\"go\">y</button>\n",
        "<p :class=\"[a, b]\">{{ a + b * c }}</p><p>text</p>\n",
    );
    let found = |selector: &str| {
        reported(&[(selector, None)], template)
            .into_iter()
            .map(|(text, _)| text)
            .collect::<Vec<_>>()
    };
    assert_eq!(found("VForExpression > Identifier.right"), ["items"]);
    assert_eq!(
        found("VOnExpression > ExpressionStatement:first-child"),
        ["run();"]
    );
    assert_eq!(
        found("VExpressionContainer > Identifier"),
        ["index", "item", "go"]
    );
    assert_eq!(found("ArrayExpression > Identifier + Identifier"), ["b"]);
    assert_eq!(found("ArrayExpression > :nth-child(2)"), ["b"]);
    assert_eq!(
        found("BinaryExpression[operator='+'] BinaryExpression"),
        ["b * c"]
    );
    assert_eq!(
        found("BinaryExpression:has(Identifier[name=c]):not([operator='*'])"),
        ["a + b * c"]
    );
    assert_eq!(
        found("VAttribute[directive=true][key.name.name='bind']"),
        [":key=\"index\"", ":class=\"[a, b]\""]
    );
    assert_eq!(
        found("VElement[rawName=/^l/]"),
        ["<li v-for=\"(item, index) in items\" :key=\"index\">{{ item }}</li>"]
    );
    assert_eq!(found("VElement[name='p'] > VText"), ["text"]);
    assert_eq!(
        reported(&[("CallExpression", None)], "<p>{{ f() }}</p>"),
        [(
            String::from("f()"),
            String::from("Using 'CallExpression' is not allowed")
        )]
    );
}
