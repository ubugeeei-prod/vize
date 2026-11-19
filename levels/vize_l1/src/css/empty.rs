//! The fixed empty-pseudo window borrows actual original parser tokens.

use super::{CssToken, StyleIssue, StyleIssueCode, StyleSyntax};
use cssparser::Token;
use vize_l0::Span;

/// Original classname, colon and literal empty-pseudo tokens from one rule.
/// No caller-supplied selector, offsets or pseudo kind grants authority.
///
/// ```compile_fail
/// use vize_l1::css::EmptyClassStyle;
/// let _ = EmptyClassStyle { insertion: 2, pseudo: "empty" };
/// ```
#[derive(Debug, Clone, Copy)]
pub struct EmptyClassStyle<'o, 'a> {
    syntax: &'o StyleSyntax<'a>,
    name: &'o CssToken<'a>,
    colon: &'o CssToken<'a>,
    pseudo: &'o CssToken<'a>,
}
impl<'o, 'a> EmptyClassStyle<'o, 'a> {
    #[must_use]
    pub const fn syntax(self) -> &'o StyleSyntax<'a> {
        self.syntax
    }
    #[must_use]
    pub const fn class_token(self) -> &'o CssToken<'a> {
        self.name
    }
    #[must_use]
    pub const fn pseudo_tokens(self) -> (&'o CssToken<'a>, &'o CssToken<'a>) {
        (self.colon, self.pseudo)
    }
    #[must_use]
    pub const fn insertion(self) -> u32 {
        self.name.span().end
    }
    #[must_use]
    pub const fn pseudo_span(self) -> Span {
        Span::new(self.colon.span().start, self.pseudo.span().end)
    }
}

pub(super) fn observe<'o, 'a>(
    syntax: &'o StyleSyntax<'a>,
) -> Result<EmptyClassStyle<'o, 'a>, StyleIssue> {
    if let Some(issue) = syntax.issue() {
        return Err(issue);
    }
    let unsupported = || StyleIssue {
        code: StyleIssueCode::UnsupportedSelector,
        span: syntax.source().span(),
    };
    let rule = syntax.rule().ok_or_else(unsupported)?;
    let [dot, name, colon, pseudo, tail @ ..] = rule.prelude() else {
        return Err(unsupported());
    };
    let Token::Ident(classname) = name.token() else {
        return Err(unsupported());
    };
    let literal = |token: &CssToken<'_>, expected: &str| {
        syntax
            .source()
            .root_source()
            .get(token.span().start as usize..token.span().end as usize)
            == Some(expected)
    };
    if !matches!(dot.token(), Token::Delim('.'))
        || !matches!(colon.token(), Token::Colon)
        || !matches!(pseudo.token(), Token::Ident(value) if value.as_ref() == "empty")
        || !literal(name, classname.as_ref())
        || !literal(pseudo, "empty")
        || tail.len() > 1
        || tail
            .first()
            .is_some_and(|token| !matches!(token.token(), Token::WhiteSpace(_)))
    {
        return Err(unsupported());
    }
    Ok(EmptyClassStyle {
        syntax,
        name,
        colon,
        pseudo,
    })
}

#[cfg(test)]
mod tests {
    use crate::{
        container::{Vue, vue::DescriptorOptions},
        css::StyleSyntax,
        markup::SurfaceParseOptions,
    };
    use cssparser::Token;
    use vize_l0::{
        Allocator,
        config::{VueDialect, VueVersion},
    };

    fn options() -> DescriptorOptions {
        DescriptorOptions {
            version: VueVersion::V3,
            dialect: VueDialect::Vue,
            template: SurfaceParseOptions::default(),
        }
    }

    #[test]
    fn empty_receipt_borrows_actual_colon_ident_and_class_boundary_in_whole_source() {
        let source = "<!-- 雪🌸 -->\r\n<style scoped>\r\n/* original */\r\n.雪:empty \t{ content:'雪'; }</style><template><p/></template>";
        let arena = Allocator::default();
        let descriptor = Vue.observe_descriptor(&arena, source, options());
        let syntax = StyleSyntax::observe(descriptor.admitted().unwrap().styles().next().unwrap());
        assert!(syntax.simple_class().is_err());
        assert!(syntax.simple_class_list().is_err());
        let receipt = syntax.empty_class().unwrap();
        let prelude = syntax.rule().unwrap().prelude();
        assert!(core::ptr::eq(receipt.syntax(), &syntax));
        assert!(core::ptr::eq(receipt.class_token(), &prelude[1]));
        let (colon, pseudo) = receipt.pseudo_tokens();
        assert!(core::ptr::eq(colon, &prelude[2]));
        assert!(core::ptr::eq(pseudo, &prelude[3]));
        assert!(matches!(colon.token(), Token::Colon));
        assert!(matches!(pseudo.token(), Token::Ident(value) if value.as_ref() == "empty"));
        assert_eq!(
            &source[receipt.insertion() as usize - "雪".len()..receipt.insertion() as usize],
            "雪"
        );
        assert_eq!(
            &source[receipt.pseudo_span().start as usize..receipt.pseudo_span().end as usize],
            ":empty"
        );
        assert!(core::ptr::eq(syntax.source().root_source(), source));
        assert_eq!(syntax.rule().unwrap().declarations()[0].name(), "content");
        let expected = vize_l0::Span::new(colon.span().start, pseudo.span().end);
        let moved = syntax;
        assert_eq!(moved.empty_class().unwrap().pseudo_span(), expected);
    }

    #[test]
    fn unsupported_empty_spellings_keep_original_tokens_errors_and_complete_source() {
        for css in [
            ".a:hover{}",
            ".a:EMPTY{}",
            ".a::empty{}",
            ".a :empty{}",
            ".a: empty{}",
            ".a/**/:empty{}",
            ".a:/**/empty{}",
            ".a:empty/**/{}",
            ".a:empty:empty{}",
            ".a:empty,.b:empty{}",
            ".\\61:empty{}",
            ".a:em\\70 ty{}",
            ".a:empty(){}",
            ".a:empty{color:v-bind(color)}",
            ".a:empty{content:'v-bind(color)'}",
            ".a:empty{content:'v/**/-bind(color)'}",
            ".a:empty{color:red",
            ".a:empty{} .b{}",
        ] {
            let source = alloc::format!("<style scoped>{css}</style><template><p/></template>");
            let arena = Allocator::default();
            let descriptor = Vue.observe_descriptor(&arena, &source, options());
            let syntax =
                StyleSyntax::observe(descriptor.admitted().unwrap().styles().next().unwrap());
            assert!(syntax.empty_class().is_err(), "{css}");
            assert!(core::ptr::eq(
                syntax.source().root_source(),
                source.as_str()
            ));
            assert_eq!(syntax.source().source(), css);
            if let Some(issue) = syntax.issue() {
                assert_eq!(syntax.empty_class().unwrap_err(), issue);
                assert!(syntax.parser_error().is_some());
            } else {
                assert!(syntax.rule().is_some());
            }
        }
    }
}
