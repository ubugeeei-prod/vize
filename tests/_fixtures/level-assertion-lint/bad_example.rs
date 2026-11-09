// Deliberately-bad fixture for tools/commands/davinci/assertion-lint.rs (Davinci
// P0-12 "lint the linter" self-test). Never compiled — this file exercises
// the lint's detection, and tests/tooling/davinci-assertion-lint.test.ts
// pins the exact finding set: lines marked FLAG must be reported with the
// bracketed category, everything else must stay silent.

pub fn render(input: &str) -> String {
    // Not test code: `.contains(` outside #[cfg(test)] must not be flagged.
    if input.contains("never-flagged") {
        return String::new();
    }
    input.to_uppercase()
}

#[cfg(test)]
mod tests {
    use super::render;

    #[test]
    fn weak_substring_assertion() {
        let output = render("x");
        assert!(output.contains("x")); // FLAG [contains]
    }

    #[test]
    fn weak_multiline_assertion() {
        let output = render("z");
        assert!(
            output.starts_with("Z"), // FLAG [starts-with]
            "prefix probes are banned by the assurance doctrine",
        );
        assert!(!output.ends_with("q")); // FLAG [ends-with]
    }

    #[test]
    fn weak_regex_assertion() {
        let output = render("w");
        assert!(Regex::new("^W$").unwrap().is_match(&output)); // FLAG [regex]
    }

    #[test]
    fn weak_partial_json_assertion() {
        let output = render("{}");
        assert!(output.contains(&serde_json::json!({ "k": 1 }).to_string())); // FLAG [partial-json]
    }

    #[test]
    fn silent_cases() {
        let output = render("y");
        // A commented-out weak assertion must not be flagged:
        // assert!(output.contains("y"));
        let quoted = "assert!(output.contains(\"in-a-string\"))";
        assert_eq!(quoted.len(), 39);
        let outside_span = output.contains("Y");
        assert_eq!(outside_span, true);
        assert_eq!(output, "Y"); // the doctrine-approved shape: exact equality
    }
}
