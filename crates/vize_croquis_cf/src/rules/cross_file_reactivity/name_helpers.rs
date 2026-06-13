use vize_carton::String;

pub(super) fn prop_names_match(left: &str, right: &str) -> bool {
    left == right || to_camel_case(left) == to_camel_case(right)
}

pub(super) fn component_names_match(left: &str, right: &str) -> bool {
    left == right || to_pascal_case(left) == to_pascal_case(right)
}

fn to_pascal_case(s: &str) -> String {
    s.split('-')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                None => String::default(),
                Some(c) => c.to_uppercase().chain(chars).collect(),
            }
        })
        .collect()
}

fn to_camel_case(s: &str) -> String {
    let mut parts = s.split('-');
    let mut out = String::from(parts.next().unwrap_or_default());

    for part in parts {
        let mut chars = part.chars();
        if let Some(first) = chars.next() {
            out.extend(first.to_uppercase());
            out.push_str(chars.as_str());
        }
    }

    out
}
