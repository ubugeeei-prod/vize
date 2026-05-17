use vize_carton::{SmallVec, String};

pub fn strip_css_comments_for_scoped(css: &str) -> String {
    if !css.contains("/*") {
        return String::from(css);
    }

    let bytes = css.as_bytes();
    let mut output = String::with_capacity(css.len());
    let mut copy_start = 0usize;
    let mut index = 0usize;
    let mut changed = false;

    while index < bytes.len() {
        match bytes[index] {
            b'"' | b'\'' => {
                let quote = bytes[index];
                index += 1;
                while index < bytes.len() {
                    let byte = bytes[index];
                    if byte == b'\\' {
                        index = (index + 2).min(bytes.len());
                        continue;
                    }
                    index += 1;
                    if byte == quote {
                        break;
                    }
                }
            }
            b'/' if bytes.get(index + 1) == Some(&b'*') => {
                output.push_str(&css[copy_start..index]);
                output.push_str("  ");
                index += 2;
                while index < bytes.len() {
                    if bytes[index] == b'*' && bytes.get(index + 1) == Some(&b'/') {
                        output.push_str("  ");
                        index += 2;
                        break;
                    }
                    output.push(if bytes[index] == b'\n' { '\n' } else { ' ' });
                    index += 1;
                }
                copy_start = index;
                changed = true;
            }
            _ => index += 1,
        }
    }

    if !changed {
        return String::from(css);
    }

    output.push_str(&css[copy_start..]);
    output
}

pub fn wrap_scoped_preprocessor_style(
    content: &str,
    scoped: Option<&str>,
    lang: Option<&str>,
) -> String {
    let Some(scoped) = scoped else {
        return String::from(content);
    };
    let Some(lang) = lang else {
        return String::from(content);
    };
    if lang == "css" {
        return String::from(content);
    }

    let mut hoisted: SmallVec<[&str; 4]> = SmallVec::new();
    let mut body: Vec<&str> = Vec::new();

    for line in content.split('\n') {
        let trimmed = line.trim_start();
        if trimmed.starts_with("@use ")
            || trimmed.starts_with("@forward ")
            || trimmed.starts_with("@import ")
        {
            hoisted.push(line);
        } else {
            body.push(line);
        }
    }

    let mut output = String::with_capacity(content.len() + scoped.len() + 8);
    if !hoisted.is_empty() {
        push_joined_lines(&mut output, &hoisted);
        output.push_str("\n\n");
    }
    output.push('[');
    output.push_str(scoped);
    output.push_str("] {\n");
    push_joined_lines(&mut output, &body);
    output.push_str("\n}");
    output
}

fn push_joined_lines(output: &mut String, lines: &[&str]) {
    for (index, line) in lines.iter().enumerate() {
        if index > 0 {
            output.push('\n');
        }
        output.push_str(line);
    }
}
