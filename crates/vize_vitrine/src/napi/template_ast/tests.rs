use serde_json::{Value, json};

use super::{TemplateAstOptions, template_ast_json};

fn parse_with(source: &str, options: &TemplateAstOptions) -> Value {
    let bytes = template_ast_json(source, options).expect("the source fits the coordinate range");
    let value: Value = serde_json::from_slice(&bytes).expect("the writer emits valid JSON");
    assert_locations(source, &value["ast"]);
    value
}

fn parse(source: &str) -> Value {
    parse_with(source, &TemplateAstOptions::default())
}

/// The UTF-16 slice a location names, as JavaScript's `String#slice` reads it.
fn slice(source: &str, loc: &Value) -> String {
    let units: Vec<u16> = source.encode_utf16().collect();
    let start = loc["start"]["offset"].as_u64().expect("start offset") as usize;
    let end = loc["end"]["offset"].as_u64().expect("end offset") as usize;
    String::from_utf16(&units[start..end]).expect("a location never splits a surrogate pair")
}

/// The 1-based line and UTF-16 column of a UTF-16 offset.
fn line_column(source: &str, offset: usize) -> (u64, u64) {
    let units: Vec<u16> = source.encode_utf16().collect();
    let before = &units[..offset];
    let line = before
        .iter()
        .filter(|unit| **unit == u16::from(b'\n'))
        .count()
        + 1;
    let line_start = before
        .iter()
        .rposition(|unit| *unit == u16::from(b'\n'))
        .map_or(0, |at| at + 1);
    (line as u64, (offset - line_start + 1) as u64)
}

/// Every location is consistent with its offsets and names the node's text.
fn assert_locations(source: &str, node: &Value) {
    match node {
        Value::Array(items) => items.iter().for_each(|item| assert_locations(source, item)),
        Value::Object(fields) => {
            for (key, value) in fields {
                if key == "loc" || key == "nameLoc" {
                    for point in ["start", "end"] {
                        let offset = value[point]["offset"].as_u64().expect("offset") as usize;
                        let expected = line_column(source, offset);
                        let actual = (
                            value[point]["line"].as_u64().expect("line"),
                            value[point]["column"].as_u64().expect("column"),
                        );
                        assert_eq!(actual, expected, "{point} of {node}");
                    }
                } else {
                    assert_locations(source, value);
                }
            }
            let text = || slice(source, &node["loc"]);
            match node["type"].as_str() {
                Some("ELEMENT") => {
                    let tag = node["tag"].as_str().expect("tag");
                    assert!(text().starts_with(&["<", tag].concat()), "{node}");
                }
                Some("COMMENT") => {
                    let content = node["content"].as_str().expect("content");
                    assert_eq!(text(), ["<!--", content, "-->"].concat());
                }
                Some("INTERPOLATION") => {
                    assert!(text().starts_with("{{") && text().ends_with("}}"));
                }
                Some("ATTRIBUTE") => {
                    let name = node["name"].as_str().expect("name");
                    assert_eq!(slice(source, &node["nameLoc"]), name);
                    assert!(text().starts_with(name));
                }
                Some("DIRECTIVE") => {
                    assert!(text().starts_with(node["rawName"].as_str().expect("rawName")));
                }
                _ => {}
            }
        }
        _ => {}
    }
}

#[test]
fn comments_stay_in_place_and_self_closing_elements_parse_without_diagnostics() {
    let result = parse("<Transition><!-- note --><div></div></Transition>");
    let transition = &result["ast"]["children"][0];
    assert_eq!(transition["tag"], "Transition");
    assert_eq!(transition["tagType"], "COMPONENT");
    assert_eq!(transition["children"][0]["type"], "COMMENT");
    assert_eq!(transition["children"][0]["content"], " note ");
    assert_eq!(transition["children"][1]["tag"], "div");
    assert_eq!(result["errors"], json!([]));

    let result = parse("<Transition><div /></Transition>");
    let div = &result["ast"]["children"][0]["children"][0];
    assert_eq!(div["tag"], "div");
    assert_eq!(div["isSelfClosing"], true);
    assert_eq!(div["children"], json!([]));
    assert_eq!(result["errors"], json!([]));
    assert_eq!(result["warnings"], json!([]));
}

#[test]
fn directives_split_into_name_argument_modifiers_and_expression() {
    let source = concat!(
        "<MyList v-for=\"(item, i) in items\" :key=\"item.id\" v-if=\"ok\">",
        "<template #row=\"{ row }\"><b @click.stop.prevent=\"go(row)\" .value=\"v\" />",
        "</template><input v-model.trim=\"text\" v-bind:[name]=\"x\" v-else></MyList>"
    );
    let result = parse(source);
    let list = &result["ast"]["children"][0];
    let props = &list["props"];
    assert_eq!(props[0]["name"], "for");
    assert_eq!(props[0]["rawName"], "v-for");
    assert_eq!(props[0]["arg"], Value::Null);
    assert_eq!(props[0]["exp"]["content"], "(item, i) in items");
    assert_eq!(slice(source, &props[0]["exp"]["loc"]), "(item, i) in items");
    assert_eq!(props[1]["name"], "bind");
    assert_eq!(props[1]["arg"]["content"], "key");
    assert_eq!(props[1]["arg"]["isStatic"], true);
    assert_eq!(props[2]["name"], "if");

    let template = &list["children"][0];
    assert_eq!(template["tagType"], "TEMPLATE");
    assert_eq!(template["props"][0]["name"], "slot");
    assert_eq!(template["props"][0]["arg"]["content"], "row");
    assert_eq!(template["props"][0]["exp"]["content"], "{ row }");

    let button = &template["children"][0];
    let click = &button["props"][0];
    assert_eq!(click["name"], "on");
    assert_eq!(click["arg"]["content"], "click");
    let modifiers: Vec<_> = click["modifiers"]
        .as_array()
        .expect("modifiers")
        .iter()
        .map(|modifier| modifier["content"].as_str().expect("content"))
        .collect();
    assert_eq!(modifiers, ["stop", "prevent"]);
    let prop = &button["props"][1];
    assert_eq!(prop["name"], "bind");
    assert_eq!(prop["modifiers"][0]["content"], "prop");
    assert_eq!(slice(source, &prop["modifiers"][0]["loc"]), ".");

    let input = &list["children"][1];
    assert_eq!(input["props"][0]["name"], "model");
    assert_eq!(input["props"][0]["modifiers"][0]["content"], "trim");
    let dynamic = &input["props"][1]["arg"];
    assert_eq!(dynamic["content"], "name");
    assert_eq!(dynamic["isStatic"], false);
    assert_eq!(slice(source, &dynamic["loc"]), "[name]");
    assert_eq!(input["props"][2]["name"], "else");
    assert_eq!(input["props"][2]["exp"], Value::Null);
}

#[test]
fn locations_count_utf16_units_across_non_ascii_text_and_lines() {
    let source = "<p title=\"日本\">\r\n  こんにちは 😀 {{ msg }}\n  <!-- é --></p>";
    let result = parse(source);
    let paragraph = &result["ast"]["children"][0];
    assert_eq!(slice(source, &paragraph["loc"]), source);
    let title = &paragraph["props"][0];
    assert_eq!(slice(source, &title["value"]["loc"]), "\"日本\"");
    assert_eq!(title["value"]["content"], "日本");
    let text = &paragraph["children"][0];
    assert_eq!(text["content"], "\r\n  こんにちは 😀 ");
    assert_eq!(slice(source, &text["loc"]), "\r\n  こんにちは 😀 ");
    let interpolation = &paragraph["children"][1];
    assert_eq!(slice(source, &interpolation["loc"]), "{{ msg }}");
    assert_eq!(interpolation["content"]["content"], "msg");
    assert_eq!(slice(source, &interpolation["content"]["loc"]), "msg");
    assert_eq!(interpolation["loc"]["start"]["line"], 2);
    assert_eq!(paragraph["children"][3]["content"], " é ");
}

#[test]
fn element_categories_follow_the_sfc_compiler() {
    let source = concat!(
        "<slot /><template v-if=\"a\" /><template /><my-thing /><component :is=\"c\" />",
        "<div is=\"vue:card\" /><transition /><svg><path /></svg>",
        "<div v-pre :a=\"b\"><Comp @x=\"y\">{{ raw }}</Comp></div>"
    );
    let result = parse(source);
    let children = result["ast"]["children"].as_array().expect("children");
    let categories: Vec<_> = children
        .iter()
        .map(|child| child["tagType"].as_str().expect("tagType"))
        .collect();
    assert_eq!(
        categories,
        [
            "SLOT",
            "TEMPLATE",
            "ELEMENT",
            "COMPONENT",
            "COMPONENT",
            "COMPONENT",
            "COMPONENT",
            "ELEMENT",
            "ELEMENT"
        ]
    );
    let pre = &children[8];
    assert_eq!(pre["props"][0]["type"], "DIRECTIVE");
    assert_eq!(pre["props"][0]["name"], "pre");
    assert_eq!(pre["props"][1]["type"], "ATTRIBUTE");
    assert_eq!(pre["props"][1]["name"], ":a");
    let comp = &pre["children"][0];
    assert_eq!(comp["tagType"], "ELEMENT");
    assert_eq!(comp["props"][0]["type"], "ATTRIBUTE");
    assert_eq!(comp["children"][0]["type"], "TEXT");
    assert_eq!(comp["children"][0]["content"], "{{ raw }}");

    let custom = parse_with(
        "<my-thing />",
        &TemplateAstOptions {
            custom_elements: Some(vec!["my-*".to_owned()]),
            ..TemplateAstOptions::default()
        },
    );
    assert_eq!(custom["ast"]["children"][0]["tagType"], "ELEMENT");
}

#[test]
fn character_references_decode_outside_raw_text() {
    let source = "<p title=\"a&amp;b\" :x=\"a &lt; b\">&lt;x&gt; {{ a &amp;&amp; b }}</p><style>a&amp;</style>";
    let result = parse(source);
    let paragraph = &result["ast"]["children"][0];
    assert_eq!(paragraph["props"][0]["value"]["content"], "a&b");
    assert_eq!(paragraph["props"][1]["exp"]["content"], "a < b");
    assert_eq!(paragraph["children"][0]["content"], "<x> ");
    assert_eq!(paragraph["children"][1]["content"]["content"], "a && b");
    assert_eq!(
        result["ast"]["children"][1]["children"][0]["content"],
        "a&amp;"
    );
}

#[test]
fn syntax_errors_are_returned_with_the_recovered_tree() {
    let source = "<div><span>text</div>\u{1}</b>";
    let result = parse(source);
    let errors = result["errors"].as_array().expect("errors");
    assert!(!errors.is_empty(), "{result}");
    for error in errors {
        assert!(
            error["code"]
                .as_str()
                .expect("code")
                .starts_with("compiler/")
        );
        assert!(!error["message"].as_str().expect("message").is_empty());
    }
    let div = &result["ast"]["children"][0];
    assert_eq!(div["tag"], "div");
    assert_eq!(div["children"][0]["tag"], "span");
    assert_eq!(slice(source, &div["children"][0]["loc"]), "<span>text");
}

#[test]
fn json_strings_escape_quotes_backslashes_and_controls() {
    let source = "<p title='\"\\'>a\tb\u{7}</p>";
    let result = parse(source);
    let paragraph = &result["ast"]["children"][0];
    assert_eq!(paragraph["props"][0]["value"]["content"], "\"\\");
    assert_eq!(paragraph["children"][0]["content"], "a\tb\u{7}");
}

#[test]
fn an_empty_template_is_an_empty_root() {
    let result = parse("");
    assert_eq!(result["ast"]["children"], json!([]));
    assert_eq!(result["ast"]["loc"]["end"]["offset"], 0);
}
