//! Emit a genuine source-owned plain CSS family; no legacy compiler helper.

use vize_l0::Span;
use vize_l1::{container::vue::StyleView, css::StyleSyntax};
use vize_l4::module::ScopeId;
use vize_l4::write::{EmitDocument, LinkSink};

use super::NativeSfcCompileError;

/// Whether an authored `lang` attribute value selects plain CSS. A bare
/// `lang` (no value) is refused like any other unproven attribute.
pub(super) fn is_css_lang_value(value: Option<&str>) -> bool {
    value.is_some_and(|value| vize_l1::container::lang::is_css_style(Some(value)))
}

/// Private evidence that this original style needs no unavailable transform.
struct PlainStyle<'o, 'a> {
    original: StyleView<'o, 'a>,
}

impl<'o, 'a> PlainStyle<'o, 'a> {
    fn observe(original: StyleView<'o, 'a>) -> Result<Self, NativeSfcCompileError> {
        if original
            .attrs()
            .iter()
            .any(|attr| attr.name != "lang" || !is_css_lang_value(attr.value))
        {
            return Err(NativeSfcCompileError::StyleCompilationUnavailable {
                container_index: original.container_index(),
                span: original.block().span(),
            });
        }
        if let Some(span) = unproven_bind(original.block().source(), original.block().start()) {
            return Err(NativeSfcCompileError::StyleBindSyntaxUnproven {
                container_index: original.container_index(),
                span,
            });
        }
        Ok(Self { original })
    }

    fn append(&self, output: &mut EmitDocument, trim: bool) {
        let block = self.original.block();
        let raw = block.source();
        let (text, prefix) = if trim {
            (raw.trim(), raw.len() - raw.trim_start().len())
        } else {
            (raw, 0)
        };
        // Match the complete SFC CSS join, including an empty trailing block.
        if !output.is_empty() {
            output.push_str("\n");
        }
        let mut at = block.start() + prefix as u32;
        for line in text.split_inclusive('\n') {
            let end = at + line.len() as u32;
            output.push_linked(line, Span::new(at, end));
            at = end;
        }
    }
}

pub(crate) fn emit<'o, 'a: 'o, L: LinkSink>(
    #[cfg(not(feature = "native-scoped-scss"))]
    descriptor: vize_l1::container::vue::AdmittedDescriptor<'o, 'a>,
    #[cfg(feature = "native-scoped-scss")] descriptor: impl Into<
        vize_l1::container::vue::CheckedStyleBlocks<'o, 'a>,
    >,
    syntax: &[StyleSyntax<'_>],
    scope_id: Option<ScopeId<'_>>,
    trim: bool,
    #[cfg(feature = "native-sfc-scss")] scss: Option<
        &mut dyn Iterator<Item = &crate::scss::ScssProducts<'_>>,
    >,
    #[cfg(feature = "native-scoped-scss")] scoped_scss: bool,
) -> Result<Option<EmitDocument>, NativeSfcCompileError> {
    #[cfg(feature = "native-scoped-scss")]
    let descriptor = descriptor.into();
    let mut output = EmitDocument::new(L::RECORDING);
    #[cfg(feature = "native-sfc-scss")]
    let enabled = scss.is_some();
    #[cfg(feature = "native-sfc-scss")]
    let mut prepared = scss.into_iter().flatten().peekable();
    for original in descriptor.styles() {
        #[cfg(feature = "native-sfc-scss")]
        let is_scss = |lang: &vize_l1::container::BlockAttr<'_>| {
            lang.name == "lang"
                && vize_l1::container::lang::style_lang(lang.value)
                    == Ok(vize_l1::css::lang::StyleLanguage::Scss)
        };
        #[cfg(feature = "native-scoped-scss")]
        let scoped = match original.attrs() {
            [a, b]
                if scoped_scss
                    && ((a.name == "scoped" && is_scss(b))
                        || (b.name == "scoped" && is_scss(a))) =>
            {
                scope_id
            }
            _ => None,
        };
        #[cfg(feature = "native-sfc-scss")]
        let accepted = matches!(original.attrs(), [lang] if is_scss(lang));
        #[cfg(feature = "native-scoped-scss")]
        let accepted = accepted || scoped.is_some();
        #[cfg(feature = "native-sfc-scss")]
        if enabled && accepted {
            use vize_l1::css::lang::StyleLanguage;
            use vize_l2::lang::scss::ScssProgram;
            let block = original.block();
            let lowered;
            let program = if let Some(first) = prepared.next_if(|p| {
                core::ptr::eq(p.block().root_source(), block.root_source())
                    && p.block().span() == block.span()
            }) {
                first.program()
            } else {
                lowered = StyleLanguage::Scss.with_original_structure(block, ScssProgram::lower);
                lowered.as_ref().map_err(|issue| *issue)
            }
            .map_err(|issue| NativeSfcCompileError::Scss {
                container_index: original.container_index(),
                issue,
            })?;
            if !output.is_empty() {
                output.push_str("\n");
            }
            #[cfg(feature = "native-scoped-scss")]
            if let Some(id) = scoped {
                vize_l4::targets::scss::append_scoped(&mut output, program, id, trim);
                continue;
            }
            vize_l4::targets::scss::append(&mut output, program);
            continue;
        }
        if original.attrs().iter().any(|attr| attr.name == "scoped") {
            super::scoped::append(original, syntax, scope_id, &mut output, trim)?;
        } else {
            PlainStyle::observe(original)?.append(&mut output, trim);
        }
    }
    #[cfg(feature = "native-sfc-scss")]
    if let Some(unused) = prepared.next() {
        return Err(NativeSfcCompileError::UnusedPreparedScss {
            span: unused.block().span(),
        });
    }
    Ok((!output.is_empty()).then_some(output))
}

/// This conservative absence proof is not a CSS parser. Escapes, raw binding
/// spelling even in strings/comments, comment-bridged spelling and unfinished
/// comments are unproven. No source bytes are decoded, copied or normalized.
fn unproven_bind(source: &str, start: u32) -> Option<Span> {
    let bytes = source.as_bytes();
    let marker = b"v-bind";
    let (mut at, mut matched, mut token_start) = (0, 0, 0);
    let mut comment_start = None;
    while let Some(&byte) = bytes.get(at) {
        if byte == b'\\' {
            return Some(Span::new(start + at as u32, start + at as u32 + 1));
        }
        if bytes
            .get(at..at + marker.len())
            .is_some_and(|raw| raw.eq_ignore_ascii_case(marker))
        {
            return Some(Span::new(
                start + at as u32,
                start + at as u32 + marker.len() as u32,
            ));
        }
        if comment_start.is_some() {
            if byte == b'*' && bytes.get(at + 1) == Some(&b'/') {
                comment_start = None;
                at += 2;
            } else {
                at += 1;
            }
            continue;
        }
        if byte == b'/' && bytes.get(at + 1) == Some(&b'*') {
            comment_start = Some(at);
            at += 2;
            continue;
        }
        if marker
            .get(matched)
            .is_some_and(|expected| byte.eq_ignore_ascii_case(expected))
        {
            if matched == 0 {
                token_start = at;
            }
            matched += 1;
            if matched == marker.len() {
                return Some(Span::new(start + token_start as u32, start + at as u32 + 1));
            }
        } else if byte.eq_ignore_ascii_case(&b'v') {
            matched = 1;
            token_start = at;
        } else {
            matched = 0;
        }
        at += 1;
    }
    comment_start.map(|at| Span::new(start + at as u32, start + source.len() as u32))
}
