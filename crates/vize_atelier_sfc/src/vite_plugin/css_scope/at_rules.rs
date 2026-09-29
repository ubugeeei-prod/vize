/// At-rules whose bodies contain selectors that require the SFC scope.
pub(super) fn should_recurse_at_rule(statement: &str) -> bool {
    matches!(
        statement.split_whitespace().next(),
        Some(
            "@container"
                | "@document"
                | "@-moz-document"
                | "@layer"
                | "@media"
                | "@scope"
                | "@supports"
        )
    )
}
