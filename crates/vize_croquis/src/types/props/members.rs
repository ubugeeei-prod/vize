use vize_carton::String;

use super::{TypeProperty, TypeResolver};

#[derive(Clone, Copy)]
enum Mode {
    Code { template_braces: Option<usize> },
    Quote { delimiter: char, escaped: bool },
    Template { escaped: bool },
}

const ROOT_MODE: Mode = Mode::Code {
    template_braces: None,
};

pub(super) fn parse(resolver: &TypeResolver, content: &str) -> Vec<TypeProperty> {
    let mut properties = Vec::new();
    let mut depth = 0;
    let mut current = String::default();
    let mut modes = vec![ROOT_MODE];
    let mut chars = content.chars().peekable();
    let mut in_line_comment = false;
    let mut in_block_comment = false;

    while let Some(character) = chars.next() {
        if modes.is_empty() {
            modes.push(ROOT_MODE);
        }
        match modes.last().copied().unwrap_or(ROOT_MODE) {
            Mode::Quote { delimiter, escaped } => {
                current.push(character);
                if escaped {
                    if let Some(mode) = modes.last_mut() {
                        *mode = Mode::Quote {
                            delimiter,
                            escaped: false,
                        };
                    }
                } else if character == '\\' {
                    if let Some(mode) = modes.last_mut() {
                        *mode = Mode::Quote {
                            delimiter,
                            escaped: true,
                        };
                    }
                } else if character == delimiter {
                    modes.pop();
                }
                continue;
            }
            Mode::Template { escaped } => {
                current.push(character);
                if escaped {
                    if let Some(mode) = modes.last_mut() {
                        *mode = Mode::Template { escaped: false };
                    }
                } else if character == '\\' {
                    if let Some(mode) = modes.last_mut() {
                        *mode = Mode::Template { escaped: true };
                    }
                } else if character == '`' {
                    modes.pop();
                } else if character == '$'
                    && chars.peek() == Some(&'{')
                    && let Some(opening) = chars.next()
                {
                    current.push(opening);
                    modes.push(Mode::Code {
                        template_braces: Some(1),
                    });
                }
                continue;
            }
            Mode::Code { template_braces } => {
                if in_line_comment {
                    if character == '\n' {
                        in_line_comment = false;
                        // The newline belongs to the comment. It separates
                        // members only when the property's type is already
                        // complete and the next token does not continue a
                        // union. `| // cats \n Cat` is still one member.
                        let keep = comment_keeps_member(template_braces, depth, &current, &chars);
                        if keep {
                            if !current.ends_with(|ch: char| ch.is_whitespace()) {
                                current.push(' ');
                            }
                        } else {
                            push_property(resolver, &mut current, &mut properties);
                        }
                    }
                    continue;
                }
                if in_block_comment {
                    if character == '*' && chars.peek() == Some(&'/') {
                        chars.next();
                        in_block_comment = false;
                        current.push(' ');
                    }
                    continue;
                }
                if character == '/' {
                    match chars.peek() {
                        Some('/') => {
                            chars.next();
                            in_line_comment = true;
                            continue;
                        }
                        Some('*') => {
                            chars.next();
                            in_block_comment = true;
                            continue;
                        }
                        _ => {}
                    }
                }
                if matches!(character, '\'' | '"') {
                    current.push(character);
                    modes.push(Mode::Quote {
                        delimiter: character,
                        escaped: false,
                    });
                    continue;
                }
                if character == '`' {
                    current.push(character);
                    modes.push(Mode::Template { escaped: false });
                    continue;
                }
                if let Some(template_braces) = template_braces {
                    current.push(character);
                    if character == '{' {
                        if let Some(mode) = modes.last_mut() {
                            *mode = Mode::Code {
                                template_braces: Some(template_braces + 1),
                            };
                        }
                    } else if character == '}' && template_braces == 1 {
                        modes.pop();
                    } else if character == '}'
                        && let Some(mode) = modes.last_mut()
                    {
                        *mode = Mode::Code {
                            template_braces: Some(template_braces - 1),
                        };
                    }
                    continue;
                }
            }
        }

        match character {
            '{' | '<' | '(' | '[' => {
                depth += 1;
                current.push(character);
            }
            '}' | ')' | ']' => {
                depth -= 1;
                current.push(character);
            }
            '>' if !current.ends_with('=') => {
                depth -= 1;
                current.push(character);
            }
            ',' | ';' | '\n' if depth == 0 => {
                // A leading `|` / `&` on the next line continues this property.
                // Flushing at the newline stored `inputmode?:` as an empty type
                // and dropped the union members.
                if character == '\n' && union_continues(&chars) {
                    if !current.ends_with(|ch: char| ch.is_whitespace()) {
                        current.push(' ');
                    }
                } else {
                    push_property(resolver, &mut current, &mut properties);
                }
            }
            _ => current.push(character),
        }
    }

    push_property(resolver, &mut current, &mut properties);
    properties
}

fn comment_keeps_member(
    template_braces: Option<usize>,
    depth: i32,
    current: &str,
    chars: &std::iter::Peekable<std::str::Chars<'_>>,
) -> bool {
    template_braces.is_some() || depth != 0 || member_awaits_type(current) || union_continues(chars)
}

/// A comment newline is not a member separator while the property is still
/// waiting for its type, including the operand of a trailing `|` or `&`.
fn member_awaits_type(current: &str) -> bool {
    let trimmed = current.trim_end();
    trimmed.is_empty() || trimmed.ends_with(['|', '&', ':', '?'])
}

fn union_continues(chars: &std::iter::Peekable<std::str::Chars<'_>>) -> bool {
    let mut look = chars.clone();
    loop {
        match look.next() {
            None => return false,
            Some(character) if character.is_whitespace() => {}
            Some('/') if look.peek() == Some(&'/') => {
                look.next();
                for character in look.by_ref() {
                    if character == '\n' {
                        break;
                    }
                }
            }
            Some('/') if look.peek() == Some(&'*') => {
                look.next();
                while let Some(character) = look.next() {
                    if character == '*' && look.peek() == Some(&'/') {
                        look.next();
                        break;
                    }
                }
            }
            Some('|' | '&') => return true,
            _ => return false,
        }
    }
}

fn push_property(
    resolver: &TypeResolver,
    current: &mut String,
    properties: &mut Vec<TypeProperty>,
) {
    if let Some(property) = resolver.parse_single_property(current) {
        properties.push(property);
    }
    current.clear();
}
