//! Lossless byte-span Vue SFC block splitter for opt-in stage capture.
//! Full product SFC descriptor migration remains #6837.

use vize_l0::{Allocator, Span, Vec};

use super::{Block, Container, ContainerError, ContainerErrorCode, ContainerFormat};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Vue;

mod scan;

use scan::{find_bytes, find_close, find_template_close, read_open};

impl ContainerFormat for Vue {
    fn split<'a>(&self, allocator: &'a Allocator, source: &'a str) -> Container<'a> {
        let mut result = Container {
            source,
            blocks: Vec::new_in(&allocator),
            errors: Vec::new_in(&allocator),
        };
        if source.len() > u32::MAX as usize {
            result.errors.push(ContainerError {
                code: ContainerErrorCode::SourceTooLarge,
                offset: 0,
            });
            return result;
        }
        let bytes = source.as_bytes();
        let (mut at, mut template, mut script, mut setup) = (0, false, false, false);
        while at < bytes.len() {
            if bytes[at] != b'<' {
                at += 1;
                continue;
            }
            if bytes[at..].starts_with(b"<!--") {
                at = find_bytes(bytes, at + 4, b"-->").map_or(bytes.len(), |end| end + 3);
                continue;
            }
            let open = match read_open(allocator, source, at) {
                Ok(Some(open)) => open,
                Ok(None) => {
                    at += 1;
                    continue;
                }
                Err(()) => {
                    result.errors.push(ContainerError {
                        code: ContainerErrorCode::UnterminatedOpenTag,
                        offset: at as u32,
                    });
                    break;
                }
            };
            let duplicate = if open.name.eq_ignore_ascii_case("template") {
                let old = template;
                template = true;
                old
            } else if open.name.eq_ignore_ascii_case("script") {
                if open.attrs.iter().any(|attr| attr.name == "setup") {
                    let old = setup;
                    setup = true;
                    old
                } else {
                    let old = script;
                    script = true;
                    old
                }
            } else {
                false
            };
            if duplicate {
                result.errors.push(ContainerError {
                    code: ContainerErrorCode::DuplicateBlock,
                    offset: at as u32,
                });
            }
            let (content_end, close_end, uncertain) = if open.self_closing {
                (open.end, Some(open.end), false)
            } else if open.name.eq_ignore_ascii_case("template") {
                find_template_close(allocator, source, open.end)
            } else {
                find_close(bytes, open.end, open.name)
                    .map_or((bytes.len(), None, false), |(start, end)| {
                        (start, Some(end), false)
                    })
            };
            if uncertain {
                result.errors.push(ContainerError {
                    code: ContainerErrorCode::UncertainInterpolation,
                    offset: at as u32,
                });
            }
            if close_end.is_none() {
                result.errors.push(ContainerError {
                    code: ContainerErrorCode::MissingCloseTag,
                    offset: at as u32,
                });
            }
            result.blocks.push(Block {
                name: open.name,
                open_tag: Span::new(at as u32, open.end as u32),
                attrs: open.attrs,
                content: Span::new(open.end as u32, content_end as u32),
                close_tag: close_end.map(|end| Span::new(content_end as u32, end as u32)),
            });
            at = close_end.unwrap_or(bytes.len());
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_blocks_with_authored_byte_spans() {
        let source = "<!-- head -->\n<template lang='pug'>\n  p 日本語\n</template>\n<script setup lang=ts>const x = 1</script>\n<style scoped>.x { color: red }</style>";
        let allocator = Allocator::default();
        let result = Vue.split(&allocator, source);
        assert!(result.errors.is_empty(), "{:?}", result.errors);
        assert_eq!(result.blocks.len(), 3);
        assert_eq!(result.blocks[0].attr("lang").unwrap().value, Some("pug"));
        assert_eq!(result.blocks[0].content.slice(source), "\n  p 日本語\n");
        assert_eq!(result.blocks[1].attr("setup").unwrap().value, None);
        assert_eq!(result.blocks[1].content.slice(source), "const x = 1");
        assert_eq!(
            result.blocks[2].close_tag.unwrap().slice(source),
            "</style>"
        );
    }

    #[test]
    fn nested_template_and_interpolation_do_not_end_root() {
        let source = "<template><template #default><p :x='\"</template>\"'>{{ '</template>' }}</p></template><div>ok</div></template>";
        let allocator = Allocator::default();
        let result = Vue.split(&allocator, source);
        assert!(result.errors.is_empty(), "{:?}", result.errors);
        assert_eq!(result.blocks.len(), 1);
        assert!(
            result.blocks[0]
                .content
                .slice(source)
                .ends_with("<div>ok</div>")
        );
        assert_eq!(
            result.blocks[0].close_tag.unwrap().slice(source),
            "</template>"
        );
    }

    #[test]
    fn nested_js_braces_do_not_end_interpolation_or_template() {
        let source =
            "<template>{{ ({ x: {} }) ? '</template>' : 'ok' }}</template><style>.ok {}</style>";
        let allocator = Allocator::default();
        let result = Vue.split(&allocator, source);
        assert!(result.errors.is_empty(), "{:?}", result.errors);
        assert_eq!(result.blocks.len(), 2);
        assert_eq!(
            result.blocks[0].content.slice(source),
            "{{ ({ x: {} }) ? '</template>' : 'ok' }}"
        );
        assert_eq!(
            result.blocks[0].close_tag.unwrap().slice(source),
            "</template>"
        );
        assert_eq!(result.blocks[1].content.slice(source), ".ok {}");
    }

    #[test]
    fn js_comment_delimiters_do_not_end_interpolation_or_template() {
        let source = "<template>{{ /* }} <template> */ value }}<div>after</div></template><style>.ok {}</style>";
        let allocator = Allocator::default();
        let result = Vue.split(&allocator, source);
        assert!(result.errors.is_empty(), "{:?}", result.errors);
        assert_eq!(result.blocks.len(), 2);
        assert_eq!(
            result.blocks[0].content.slice(source),
            "{{ /* }} <template> */ value }}<div>after</div>"
        );
        assert_eq!(
            result.blocks[0].close_tag.unwrap().slice(source),
            "</template>"
        );
        assert_eq!(result.blocks[1].content.slice(source), ".ok {}");
    }

    #[test]
    fn js_regex_delimiters_do_not_end_interpolation() {
        let source = "<template>{{ /}} <template>/.test(value) ? count / 2 : 0 }}<div>after</div></template>";
        let allocator = Allocator::default();
        let result = Vue.split(&allocator, source);
        assert!(result.errors.is_empty(), "{:?}", result.errors);
        assert_eq!(result.blocks.len(), 1);
        assert_eq!(
            result.blocks[0].content.slice(source),
            "{{ /}} <template>/.test(value) ? count / 2 : 0 }}<div>after</div>"
        );
    }

    #[test]
    fn nested_template_expression_is_reported_as_uncertain() {
        let source = "<template>{{ `value ${name}` }}</template>";
        let allocator = Allocator::default();
        let result = Vue.split(&allocator, source);
        assert_eq!(result.blocks.len(), 1);
        assert_eq!(
            result.errors[0].code,
            ContainerErrorCode::UncertainInterpolation
        );
        assert_eq!(result.errors[1].code, ContainerErrorCode::MissingCloseTag);
        assert!(result.blocks[0].close_tag.is_none());
    }

    #[test]
    fn duplicate_and_missing_closer_report_offsets() {
        let source = "<template>x</template><template>y";
        let allocator = Allocator::default();
        let result = Vue.split(&allocator, source);
        assert_eq!(result.blocks.len(), 2);
        assert_eq!(result.errors.len(), 2);
        assert_eq!(result.errors[0].code, ContainerErrorCode::DuplicateBlock);
        assert_eq!(result.errors[1].code, ContainerErrorCode::MissingCloseTag);
        assert_eq!(result.blocks[1].content.slice(source), "y");
    }

    #[test]
    fn unclosed_interpolation_is_not_a_trusted_capture_boundary() {
        let source = "<template>{{ '</template>' </template>";
        let allocator = Allocator::default();
        let result = Vue.split(&allocator, source);
        assert!(
            result
                .errors
                .iter()
                .any(|error| error.code == ContainerErrorCode::UncertainInterpolation)
        );
    }

    #[test]
    fn long_closed_interpolation_has_no_artificial_limit() {
        let mut source = alloc::string::String::from("<template>{{ ");
        source.push_str(&"x".repeat(8192));
        source.push_str(" }}</template>");
        let allocator = Allocator::default();
        let result = Vue.split(&allocator, &source);
        assert!(result.errors.is_empty(), "{:?}", result.errors);
        assert_eq!(result.blocks.len(), 1);
        assert_eq!(
            result.blocks[0].close_tag.unwrap().slice(&source),
            "</template>"
        );
    }
}
