//! Allocation-free lexical helpers for classifying template expressions.
//!
//! This intentionally skips quoted and commented text without attempting to
//! parse JavaScript. Callers only need structural delimiters and arrows.

struct StructuralBytes<'a> {
    bytes: &'a [u8],
    index: usize,
}

impl<'a> StructuralBytes<'a> {
    fn new(input: &'a str, index: usize) -> Self {
        Self {
            bytes: input.as_bytes(),
            index,
        }
    }
}

impl Iterator for StructuralBytes<'_> {
    type Item = (usize, u8);

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < self.bytes.len() {
            let index = self.index;
            let byte = self.bytes[index];
            if byte.is_ascii_whitespace() {
                self.index += 1;
                continue;
            }
            if matches!(byte, b'\'' | b'"' | b'`') {
                self.index += 1;
                let mut escaped = false;
                while let Some(&quoted) = self.bytes.get(self.index) {
                    self.index += 1;
                    if escaped {
                        escaped = false;
                    } else if quoted == b'\\' {
                        escaped = true;
                    } else if quoted == byte {
                        break;
                    }
                }
                continue;
            }
            if byte == b'/' && self.bytes.get(index + 1) == Some(&b'/') {
                self.index += 2;
                while self
                    .bytes
                    .get(self.index)
                    .is_some_and(|byte| *byte != b'\n')
                {
                    self.index += 1;
                }
                continue;
            }
            if byte == b'/' && self.bytes.get(index + 1) == Some(&b'*') {
                self.index += 2;
                while self.index + 1 < self.bytes.len()
                    && (self.bytes[self.index] != b'*' || self.bytes[self.index + 1] != b'/')
                {
                    self.index += 1;
                }
                self.index = (self.index + 2).min(self.bytes.len());
                continue;
            }
            self.index += 1;
            return Some((index, byte));
        }
        None
    }
}

pub(super) fn skip_js_trivia(input: &str, mut index: usize) -> usize {
    let bytes = input.as_bytes();
    loop {
        while bytes
            .get(index)
            .is_some_and(|byte| byte.is_ascii_whitespace())
        {
            index += 1;
        }
        if bytes
            .get(index..index + 2)
            .is_some_and(|trivia| trivia == b"//")
        {
            index += 2;
            while bytes.get(index).is_some_and(|byte| *byte != b'\n') {
                index += 1;
            }
            continue;
        }
        if bytes
            .get(index..index + 2)
            .is_some_and(|trivia| trivia == b"/*")
        {
            index += 2;
            while index + 1 < bytes.len() && (bytes[index] != b'*' || bytes[index + 1] != b'/') {
                index += 1;
            }
            index = (index + 2).min(bytes.len());
            continue;
        }
        return index;
    }
}

pub(super) fn matching_paren_index(input: &str, open_index: usize) -> Option<usize> {
    if input.as_bytes().get(open_index) != Some(&b'(') {
        return None;
    }

    let mut depth = 0u32;
    for (index, byte) in StructuralBytes::new(input, open_index) {
        match byte {
            b'(' => depth += 1,
            b')' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
    }
    None
}

/// Finds only an arrow that forms the outer callable. Nested callbacks in
/// parameter types, defaults, calls, arrays, and objects do not qualify.
pub(super) fn top_level_arrow_index(input: &str) -> Option<usize> {
    let bytes = input.as_bytes();
    let (mut parens, mut brackets, mut braces) = (0u32, 0u32, 0u32);
    for (index, byte) in StructuralBytes::new(input, 0) {
        match byte {
            b'(' => parens += 1,
            b')' => parens = parens.checked_sub(1)?,
            b'[' => brackets += 1,
            b']' => brackets = brackets.checked_sub(1)?,
            b'{' => braces += 1,
            b'}' => braces = braces.checked_sub(1)?,
            b'=' if bytes.get(index + 1) == Some(&b'>')
                && parens == 0
                && brackets == 0
                && braces == 0 =>
            {
                return Some(index);
            }
            _ => {}
        }
    }
    None
}

/// Cheaply recognizes a possible sequence-expression separator. If lexical
/// ambiguity makes the delimiter balance invalid, prefer the exact parser
/// fallback over rejecting a valid callback shape.
pub(super) fn has_top_level_comma(input: &str) -> bool {
    let (mut parens, mut brackets, mut braces) = (0u32, 0u32, 0u32);
    for (_, byte) in StructuralBytes::new(input, 0) {
        match byte {
            b'(' => parens += 1,
            b')' => {
                let Some(next) = parens.checked_sub(1) else {
                    return input.contains(',');
                };
                parens = next;
            }
            b'[' => brackets += 1,
            b']' => {
                let Some(next) = brackets.checked_sub(1) else {
                    return input.contains(',');
                };
                brackets = next;
            }
            b'{' => braces += 1,
            b'}' => {
                let Some(next) = braces.checked_sub(1) else {
                    return input.contains(',');
                };
                braces = next;
            }
            b',' if parens == 0 && brackets == 0 && braces == 0 => return true,
            _ => {}
        }
    }
    (parens != 0 || brackets != 0 || braces != 0) && input.contains(',')
}
