//! Machine paths are not part of the typechecker fingerprint.
//!
//! `vize check` prints the absolute project directory on stderr, and a few
//! projects repeat that directory inside diagnostic text. Two runs of the
//! same tree then hash differently. Rewriting each absolute path to `<abs>`
//! keeps that noise out of the hash. Relative paths (`src/App.vue`) and the
//! diagnostic wording stay intact, and `file_count` is compared separately.

pub fn normalize_machine_paths(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let mut index = 0;
    while index < chars.len() {
        if chars[index] == '/' && (index == 0 || !is_path_char(chars[index - 1])) {
            let start = index;
            index += 1;
            while index < chars.len() && is_path_char(chars[index]) {
                index += 1;
            }
            let mut end = index;
            if end >= start + 3
                && chars[end - 3] == '.'
                && chars[end - 2] == '.'
                && chars[end - 1] == '.'
            {
                end -= 3;
            }
            let path: String = chars[start..end].iter().collect();
            if path.matches('/').count() >= 2 {
                out.push_str("<abs>");
                index = end;
                continue;
            }
            index = start;
        }
        out.push(chars[index]);
        index += 1;
    }
    out
}

fn is_path_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '/' | '.' | '_' | '-' | '+' | '~' | '@')
}
