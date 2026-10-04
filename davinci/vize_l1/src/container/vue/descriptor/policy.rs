//! Classify each original block at its existing emission event, without rescanning.

use vize_l0::config::{VueDialect, VueVersion};
use vize_l0::{Allocator, SourceRoot, Span, Vec};

use super::super::scan::MatchedClose;
use super::{
    DescriptorIssue, DescriptorIssueCode as Code, DescriptorOptions, Selection, StyleSelection,
    TemplateNamePair, TemplateSelection,
};
use crate::{container::Block, embed::Lang};

pub(super) struct Policy<'a> {
    pub(super) root: Option<SourceRoot<'a>>,
    pub(super) issues: Vec<'a, DescriptorIssue>,
    pub(super) ordinary: Option<Selection<'a>>,
    pub(super) setup: Option<Selection<'a>>,
    pub(super) template: Option<TemplateSelection<'a>>,
    pub(super) styles: Vec<'a, StyleSelection<'a>>,
}

impl<'a> Policy<'a> {
    pub(super) fn new(
        allocator: &'a Allocator,
        source: &'a str,
        options: DescriptorOptions,
    ) -> Self {
        Self::at_version(allocator, source, options, VueVersion::V3)
    }

    pub(super) fn new_vue2(
        allocator: &'a Allocator,
        source: &'a str,
        options: DescriptorOptions,
    ) -> Self {
        Self::at_version(allocator, source, options, VueVersion::V2)
    }

    fn at_version(
        allocator: &'a Allocator,
        source: &'a str,
        options: DescriptorOptions,
        required: VueVersion,
    ) -> Self {
        let mut state = Self {
            root: SourceRoot::new(source).ok(),
            issues: Vec::new_in(&allocator),
            ordinary: None,
            setup: None,
            template: None,
            styles: Vec::new_in(&allocator),
        };
        for (unsupported, code) in [
            (options.version != required, Code::UnsupportedVersion),
            (options.dialect != VueDialect::Vue, Code::UnsupportedDialect),
            (
                options.template.experimental_in_tag_comments,
                Code::UnsupportedOptions,
            ),
        ] {
            if unsupported {
                state.issue(code, None, Span::new(0, 0));
            }
        }
        state
    }

    pub(super) fn issue(&mut self, code: Code, container_index: Option<usize>, span: Span) {
        self.issues.push(DescriptorIssue {
            code,
            container_index,
            span,
        });
    }

    pub(super) fn finish(&mut self) {
        if self.ordinary.is_none() && self.setup.is_none() && self.template.is_none() {
            self.issue(Code::MissingComponentBlock, None, Span::new(0, 0));
        }
    }

    pub(super) fn record(
        &mut self,
        index: usize,
        block: &Block<'a>,
        uncertain: bool,
        self_closing: bool,
        closing: Option<MatchedClose>,
    ) {
        let script = block.name.eq_ignore_ascii_case("script");
        let template = block.name.eq_ignore_ascii_case("template");
        let style = block.name.eq_ignore_ascii_case("style");
        if !script && !template && !style {
            self.issue(Code::UnsupportedBlock, Some(index), block.open_tag);
            return;
        }
        let expected_name = if script {
            "script"
        } else if template {
            "template"
        } else {
            "style"
        };
        if block.name != expected_name {
            self.issue(Code::UnsupportedBlockSpelling, Some(index), block.open_tag);
        }
        if uncertain || self_closing || block.close_tag.is_none() {
            self.issue(Code::UnsupportedBoundary, Some(index), block.open_tag);
        }
        let (mut lang, mut setup) = (Lang::Js, false);
        let (mut seen_lang, mut seen_setup, mut seen_src) = (false, false, false);
        let (mut seen_scoped, mut seen_module) = (false, false);
        let mut previous_end = None;
        for attr in &block.attrs {
            if let Some(end) = previous_end {
                let separator = self
                    .root
                    .and_then(|root| root.source().get(end..attr.span.start as usize));
                if !separator.is_some_and(|text| {
                    !text.is_empty() && text.bytes().all(|byte| byte.is_ascii_whitespace())
                }) {
                    self.issue(Code::AmbiguousAttribute, Some(index), attr.span);
                }
            }
            // The capture scanner includes trailing whitespace in a bare
            // attribute's span while probing for `=`. Keep that original span,
            // but measure its separator from the last authored non-space byte.
            previous_end = self.root.and_then(|root| {
                root.source()
                    .get(attr.span.start as usize..attr.span.end as usize)
                    .map(|raw| attr.span.start as usize + raw.trim_ascii_end().len())
            });
            let seen = if attr.name.eq_ignore_ascii_case("lang") {
                &mut seen_lang
            } else if attr.name.eq_ignore_ascii_case("setup") {
                &mut seen_setup
            } else if attr.name.eq_ignore_ascii_case("src") {
                &mut seen_src
            } else if style && attr.name.eq_ignore_ascii_case("scoped") {
                &mut seen_scoped
            } else if style && attr.name.eq_ignore_ascii_case("module") {
                &mut seen_module
            } else {
                self.issue(Code::UnsupportedAttribute, Some(index), attr.span);
                continue;
            };
            if *seen {
                self.issue(Code::DuplicateAttribute, Some(index), attr.span);
            }
            *seen = true;
            match attr.name {
                "lang" => match attr.value {
                    Some(value) if value.contains('&') => {
                        self.issue(Code::EncodedLanguage, Some(index), attr.span)
                    }
                    Some("js") if script => lang = Lang::Js,
                    Some("ts") if script => lang = Lang::Ts,
                    Some("html") if template => {}
                    Some(value) if style && !value.is_empty() => {}
                    _ => self.issue(Code::UnsupportedLanguage, Some(index), attr.span),
                },
                "setup" if script => {
                    setup = true;
                    if attr.value.is_some() {
                        self.issue(Code::UnsupportedSetupValue, Some(index), attr.span);
                    }
                }
                "src" => self.issue(Code::ExternalSource, Some(index), attr.span),
                "scoped" | "module" if style => {}
                _ => self.issue(Code::UnsupportedAttribute, Some(index), attr.span),
            }
        }
        let Some(content) = self.root.and_then(|root| {
            let text = root
                .source()
                .get(block.content.start as usize..block.content.end as usize)?;
            root.block(text, block.content.start).ok()
        }) else {
            self.issue(Code::InvalidSourceFrame, Some(index), block.content);
            return;
        };
        if style {
            self.styles.push(StyleSelection {
                index,
                block: content,
            });
            return;
        }
        let selected = Selection {
            index,
            block: content,
            lang,
        };
        if template {
            if self.template.is_some() {
                self.issue(Code::DuplicateRole, Some(index), block.open_tag);
            } else {
                let names = self.root.zip(closing).and_then(|(root, close)| {
                    root.whole_block()
                        .span_of(block.name)
                        .map(|opening| TemplateNamePair {
                            opening,
                            closing: close.name(),
                        })
                });
                self.template = Some(TemplateSelection {
                    selection: selected,
                    names,
                });
            }
            return;
        }
        let slot = if setup {
            &mut self.setup
        } else {
            &mut self.ordinary
        };
        if slot.is_some() {
            self.issue(Code::DuplicateRole, Some(index), block.open_tag);
        } else {
            *slot = Some(selected);
        }
        if script
            && let (Some(ordinary), Some(setup)) = (self.ordinary, self.setup)
            && ordinary.lang != setup.lang
        {
            self.issue(Code::LanguageMismatch, Some(index), block.open_tag);
        }
    }
}
