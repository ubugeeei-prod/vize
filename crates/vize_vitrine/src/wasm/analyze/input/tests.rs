use super::super::super::analyze_sfc_json_with_options;

fn analyze(source: &str) -> serde_json::Value {
    analyze_sfc_json_with_options(source, "Pattern.vue", false, true).unwrap()
}

#[test]
fn root_and_nested_patterns_retain_authored_scope_and_diagnostic_ranges() {
    for root in [true, false] {
        let header = if root {
            "<template v-match=\"result\">"
        } else {
            "<template><div v-match=\"result\">"
        };
        let footer = if root {
            "</template>"
        } else {
            "</div></template>"
        };
        let source = format!(
            "<script setup>// \u{65e5}\u{672c}\u{8a9e} \u{1f3a8}\r\nconst result = {{}};</script>\r\n{header}<p v-when=\"{{ kind: &quot;ok&quot;, const rows }} as whole if (rows.length &gt; 0)\">{{{{ rows }}}} {{{{ whole }}}}</p><p v-when=\"_\">Empty</p>{footer}"
        );
        let result = analyze(&source);
        assert_eq!(result["diagnostics"], serde_json::json!([]));
        let scopes = result["croquis"]["scopes"].as_array().unwrap();
        let arms: Vec<_> = scopes.iter().filter(|s| s["kind"] == "v-when").collect();
        assert_eq!(arms.len(), 2);
        let subject = scopes.iter().find(|s| s["kind"] == "v-match").unwrap();
        assert_eq!(arms[0]["parentIds"][0], subject["id"]);
        assert_eq!(arms[1]["parentIds"][0], subject["id"]);
        let mut bindings: Vec<_> = arms[0]["bindings"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        bindings.sort_unstable();
        assert_eq!(bindings, ["rows", "whole"]);
        let start = source.find("<p v-when").unwrap();
        let end = source.find("</p>").unwrap();
        assert_eq!(arms[0]["start"], source[..start].encode_utf16().count());
        assert_eq!(arms[0]["end"], source[..end].encode_utf16().count());
        assert_eq!(arms[0]["isTemplateScope"], true);
        let broken = source.replace("as whole", "as rows");
        let result = analyze(&broken);
        assert_eq!(result["croquis"]["stats"]["error_count"], 1);
        assert_eq!(result["diagnostics"], result["croquis"]["diagnostics"]);
        assert!(
            result["diagnostics"][0]["message"]
                .as_str()
                .unwrap()
                .contains("Duplicate")
        );
        assert_eq!(analyze(&source)["diagnostics"], serde_json::json!([]));
    }
}

#[test]
fn root_requires_explicit_opt_in_and_preserves_original_body_for_inspection() {
    let source = "<template v-match=\"subject\"><p v-when=\"_\">ok</p></template>";
    let off = analyze_sfc_json_with_options(source, "Pattern.vue", false, false).unwrap_err();
    assert!(off.contains("patternedTemplate"));
    let result = analyze(source);
    assert_eq!(result["diagnostics"], serde_json::json!([]));
    assert_eq!(result["vir"], result["folio"]["croquis"]);
    assert!(result["vir"].as_str().unwrap().contains("v-match"));
}

#[test]
fn missing_arms_are_warnings_not_silent_success() {
    let result = analyze("<template v-match=\"subject\"><p>not an arm</p></template>");
    assert_eq!(result["croquis"]["stats"]["error_count"], 0);
    assert_eq!(result["croquis"]["stats"]["warning_count"], 2);
    let diagnostics = result["diagnostics"].as_array().unwrap();
    assert_eq!(diagnostics.len(), 2);
    assert!(diagnostics.iter().all(|d| d["severity"] == "warning"));
}
