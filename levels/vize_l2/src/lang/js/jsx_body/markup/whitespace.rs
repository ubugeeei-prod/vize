//! Exactly the formatting-only family that creates no JSX text child.
//! Single-line spaces remain meaningful; other normalized values stay refused.

pub(super) fn ignored(text: &str) -> bool {
    text.contains(['\n', '\r'])
        && text
            .bytes()
            .all(|byte| matches!(byte, b' ' | b'\t' | b'\r' | b'\n'))
}
