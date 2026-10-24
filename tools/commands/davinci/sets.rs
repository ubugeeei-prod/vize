//! Set-row recipes: the element sets the tree-construction algorithm names.

use crate::html_spec::{Rule, Spec, start_tags};
use crate::recipes::{
    Row, between, dedupe, items_after, member, row, rule_tags, start_tag_rules, tags_where,
};

fn strip_article(item: &str) -> String {
    let item = item
        .strip_prefix("An ")
        .or_else(|| item.strip_prefix("A "))
        .unwrap_or(item);
    item.split(" element").next().unwrap_or(item).to_string()
}

/// The start tags an insertion mode inserts as the current node's child,
/// and the ones it first wraps in an implied element.
fn mode_children(rules: &[Rule]) -> (Vec<String>, Vec<String>) {
    let mut children = Vec::new();
    let mut wrapped = Vec::new();
    for (tags, rule) in start_tag_rules(rules) {
        if tags.is_empty() {
            continue;
        }
        let steps = &rule.steps;
        if steps.contains("Insert an HTML element for a \"") {
            wrapped.extend(tags);
        } else if steps.contains("\"hidden\"")
            && steps.contains("Insert an HTML element for the token")
        {
            children.extend(tags.into_iter().map(|tag| format!("{tag}?type-hidden")));
        } else if steps.contains("Insert an HTML element for the token")
            || steps
                .contains("Process the token using the rules for the \"in head\" insertion mode")
        {
            children.extend(tags);
        }
    }
    (children, wrapped)
}

pub fn sets(spec: &Spec) -> Result<Vec<Row>, String> {
    let stack = spec.section("the-stack-of-open-elements")?;
    let special: Vec<String> = between(stack, "special parsing rules: ", ".")?
        .split("; ")
        .flat_map(|part| {
            part.trim_start_matches("and ")
                .trim_start_matches("HTML's ")
                .split(", ")
        })
        .map(member)
        .collect();
    let scope: Vec<String> = items_after(
        stack,
        "have a particular element in scope when it has that element in the specific scope consisting of the following element types:",
    )?
    .iter()
    .map(|item| member(item))
    .collect();
    let mut button_scope = Vec::new();
    for item in items_after(
        stack,
        "in button scope when it has that element in the specific scope consisting of the following element types:",
    )? {
        if item.starts_with("All the element types listed above") {
            button_scope.extend(scope.iter().cloned());
        } else {
            button_scope.push(member(&item));
        }
    }
    let formatting = spec.section("the-list-of-active-formatting-elements")?;
    let marker: Vec<String> = between(
        formatting,
        "The markers are inserted when entering ",
        " elements",
    )?
    .split(", ")
    .map(member)
    .collect();
    let implied = spec.section("closing-elements-that-have-implied-end-tags")?;
    let implied_end: Vec<String> =
        between(implied, "while the current node is ", ", the UA must pop")?
            .split(", ")
            .map(|item| {
                strip_article(&format!(
                    "A {}",
                    item.trim_start_matches("or ")
                        .trim_start_matches("an ")
                        .trim_start_matches("a ")
                ))
            })
            .collect();

    let head = spec.rules("parsing-main-inhead")?;
    let body = spec.rules("parsing-main-inbody")?;
    let caption = spec.rules("parsing-main-incaption")?;
    let colgroup = spec.rules("parsing-main-incolgroup")?;
    let table = spec.rules("parsing-main-intable")?;
    let tbody = spec.rules("parsing-main-intbody")?;
    let tr = spec.rules("parsing-main-intr")?;
    let foreign = spec.rules("parsing-main-inforeign")?;

    let closes_p = dedupe(tags_where(&body, "p element in button scope"));
    let heading = rule_tags(&body, "\"h1\"")?;
    let li_rule = start_tag_rules(&body)
        .find(|(tags, _)| tags == &["li"])
        .map(|(_, rule)| rule)
        .ok_or("clause changed: no li rule")?;
    let loop_transparent: Vec<String> = between(
        &li_rule.steps,
        "is in the special category, but is not an ",
        " element",
    )?
    .split(", ")
    .map(|item| item.trim_start_matches("or ").to_string())
    .collect();
    let table_part = rule_tags(&caption, "\"caption\", \"col\"")?;
    let mut document_part = rule_tags(&body, "tag name is \"html\"")?;
    document_part.extend(rule_tags(&body, "tag name is \"body\"")?);
    document_part.extend(rule_tags(&body, "tag name is \"frameset\"")?);
    document_part.extend(
        rule_tags(&body, "\"frame\", \"head\"")?
            .into_iter()
            .filter(|tag| !table_part.contains(tag)),
    );

    let raw_phrases = [
        "generic RCDATA element parsing algorithm",
        "generic raw text element parsing algorithm",
        "script data state",
        "RCDATA state",
        "PLAINTEXT state",
    ];
    let mut raw_text = Vec::new();
    let mut scripting = Vec::new();
    for rules in [&head, &body] {
        for rule in rules.iter() {
            if !raw_phrases.iter().any(|phrase| rule.steps.contains(phrase)) {
                continue;
            }
            for condition in &rule.conditions {
                if condition.contains("if scripting mode is not Disabled") {
                    scripting.extend(start_tags(condition));
                } else {
                    raw_text.extend(start_tags(condition));
                }
            }
        }
    }
    let pop = "Immediately pop the current node off the stack of open elements";
    let mut void = tags_where(&body, pop);
    void.extend(tags_where(&head, pop));
    void.extend(tags_where(&colgroup, pop));

    let breakout_rule = foreign
        .iter()
        .find(|rule| {
            rule.conditions
                .iter()
                .any(|condition| condition.contains("\"b\", \"big\""))
        })
        .ok_or("clause changed: no foreign breakout rule")?;
    let mut breakout = Vec::new();
    for condition in &breakout_rule.conditions {
        let conditional = condition.contains("if the token has any attributes named");
        for tag in start_tags(condition) {
            breakout.push(if conditional {
                format!("{tag}?font-presentational")
            } else {
                tag
            });
        }
    }
    let tree = spec.section("tree-construction")?;
    let text_integration: Vec<String> = items_after(
        tree,
        "A node is a MathML text integration point if it is one of the following elements:",
    )?
    .iter()
    .map(|item| member(&strip_article(item)))
    .collect();
    let html_integration = dedupe(
        items_after(
            tree,
            "A node is an HTML integration point if it is one of the following elements:",
        )?
        .iter()
        .map(|item| {
            let name = member(&strip_article(item));
            if item.contains("\"encoding\"") {
                format!("{name}?encoding-html")
            } else {
                name
            }
        })
        .collect(),
    );

    let (table_children, table_wrapped) = mode_children(&table);
    // A mode's "anything else" defers to "in table": the table-mode children
    // no rule of the mode itself handles are inserted there too.
    let inherited = |mode: &[Rule]| -> Vec<String> {
        let handled: Vec<String> = start_tag_rules(mode).flat_map(|(tags, _)| tags).collect();
        table_children
            .iter()
            .filter(|tag| {
                !handled
                    .iter()
                    .any(|handled| tag.split('?').next() == Some(handled))
            })
            .cloned()
            .collect()
    };
    let (mut section_children, section_wrapped) = mode_children(&tbody);
    section_children.extend(inherited(&tbody));
    let (mut row_children, _) = mode_children(&tr);
    row_children.extend(inherited(&tr));
    let (colgroup_children, _) = mode_children(&colgroup);
    let transparent: Vec<String> = spec
        .elements
        .iter()
        .filter(|cells| {
            cells
                .get(4)
                .is_some_and(|model| model.contains("transparent"))
        })
        .filter_map(|cells| cells.first())
        .filter(|name| name.as_str() != "autonomous custom elements")
        .map(|name| member(name))
        .collect();

    let p = |anchor: &str| format!("parsing.html#{anchor}");
    Ok(vec![
        row("set", "special", &p("special"), special),
        row("set", "scope", &p("has-an-element-in-scope"), scope),
        row(
            "set",
            "button-scope",
            &p("has-an-element-in-button-scope"),
            button_scope,
        ),
        row("set", "marker", &p("concept-parser-marker"), marker),
        row(
            "set",
            "implied-end",
            &p("generate-implied-end-tags"),
            implied_end,
        ),
        row("set", "closes-p", &p("parsing-main-inbody"), closes_p),
        row("set", "heading", &p("parsing-main-inbody"), heading),
        row(
            "set",
            "list-item-loop-transparent",
            &p("parsing-main-inbody"),
            loop_transparent,
        ),
        row(
            "set",
            "table-part",
            &p("parsing-main-incaption"),
            table_part,
        ),
        row(
            "set",
            "document-part",
            &p("parsing-main-inbody"),
            dedupe(document_part),
        ),
        row(
            "set",
            "raw-text",
            &p("generic-raw-text-element-parsing-algorithm"),
            dedupe(raw_text),
        ),
        row(
            "set",
            "scripting-dependent",
            &p("parsing-main-inbody"),
            dedupe(scripting),
        ),
        row("set", "void", &p("parsing-main-inbody"), dedupe(void)),
        row(
            "set",
            "foreign-breakout",
            &p("parsing-main-inforeign"),
            breakout,
        ),
        row(
            "set",
            "mathml-text-integration",
            &p("mathml-text-integration-point"),
            text_integration,
        ),
        row(
            "set",
            "html-integration",
            &p("html-integration-point"),
            html_integration,
        ),
        row(
            "set",
            "table-children",
            &p("parsing-main-intable"),
            table_children,
        ),
        row(
            "set",
            "table-wrapped",
            &p("parsing-main-intable"),
            table_wrapped,
        ),
        row(
            "set",
            "section-children",
            &p("parsing-main-intbody"),
            section_children,
        ),
        row(
            "set",
            "section-wrapped",
            &p("parsing-main-intbody"),
            section_wrapped,
        ),
        row("set", "row-children", &p("parsing-main-intr"), row_children),
        row(
            "set",
            "colgroup-children",
            &p("parsing-main-incolgroup"),
            colgroup_children,
        ),
        row("set", "transparent", "dom.html#transparent", transparent),
    ])
}
