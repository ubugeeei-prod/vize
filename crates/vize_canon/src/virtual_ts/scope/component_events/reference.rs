use vize_carton::String;

use crate::virtual_ts::helpers::to_safe_identifier;

pub(super) fn component_reference_expression(name: &str) -> String {
    if name.split('.').all(|segment| {
        let mut bytes = segment.bytes();
        matches!(bytes.next(), Some(b'_' | b'$' | b'a'..=b'z' | b'A'..=b'Z'))
            && bytes.all(|byte| byte == b'_' || byte == b'$' || byte.is_ascii_alphanumeric())
    }) {
        String::from(name)
    } else {
        to_safe_identifier(name)
    }
}
