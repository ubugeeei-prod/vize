pub(super) fn delimiter_imbalance(source: &str) -> bool {
    let bytes = source.as_bytes();
    let mut index = 0;
    let mut paren = 0i32;
    let mut bracket = 0i32;
    let mut brace = 0i32;
    let mut quote: Option<u8> = None;
    while let Some(&current) = bytes.get(index) {
        if let Some(open) = quote {
            if current == b'\\' {
                index = index.saturating_add(2);
                continue;
            }
            if current == open {
                quote = None;
            }
            index = index.saturating_add(1);
            continue;
        }
        if current == b'/' {
            match bytes.get(index.saturating_add(1)) {
                Some(b'/') => {
                    index = index.saturating_add(2);
                    while let Some(&next) = bytes.get(index) {
                        index = index.saturating_add(1);
                        if next == b'\n' {
                            break;
                        }
                    }
                    continue;
                }
                Some(b'*') => {
                    index = index.saturating_add(2);
                    while let Some(&next) = bytes.get(index) {
                        if next == b'*' && bytes.get(index.saturating_add(1)) == Some(&b'/') {
                            index = index.saturating_add(2);
                            break;
                        }
                        index = index.saturating_add(1);
                    }
                    continue;
                }
                _ => {}
            }
        }
        match current {
            b'\'' | b'"' | b'`' => quote = Some(current),
            b'(' => paren += 1,
            b')' => paren -= 1,
            b'[' => bracket += 1,
            b']' => bracket -= 1,
            b'{' => brace += 1,
            b'}' => brace -= 1,
            _ => {}
        }
        if paren < 0 || bracket < 0 || brace < 0 {
            return true;
        }
        index = index.saturating_add(1);
    }
    paren != 0 || bracket != 0 || brace != 0 || quote.is_some()
}
