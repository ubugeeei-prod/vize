use vize_l0::String;

/// Strip vize disable comments from CSS source for compilation
pub fn strip_vize_comments(source: &str) -> String {
    let mut result = String::with_capacity(source.len());
    let mut rest = source;

    while let Some((before, from_comment)) =
        rest.find("/*").and_then(|at| rest.split_at_checked(at))
    {
        result.push_str(before);
        // An unterminated comment runs to the end of the source.
        let comment_len = from_comment
            .get(2..)
            .and_then(|body| body.find("*/"))
            .map_or(from_comment.len(), |end| end + 4);
        let (comment, after) = from_comment
            .split_at_checked(comment_len)
            .unwrap_or((from_comment, ""));
        // Only strip vize-related comments
        if !comment.contains("vize-disable") && !comment.contains("vize-enable") {
            result.push_str(comment);
        }
        rest = after;
    }
    result.push_str(rest);

    result
}
