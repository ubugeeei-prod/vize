//! Original whole-script syntax for tools using the stock parser options.
use oxc_parser::{Parser, ParserReturn};
use oxc_span::SourceType;
use vize_l0::{Allocator, SourceBlock};

/// A syntax reader may inspect a recovered tree and its complete diagnostics.
/// The parse profile and byte frame stay attached to the original parser result.
pub struct AuthoredProgram<'a> {
    block: SourceBlock<'a>,
    source_type: SourceType,
    parsed: ParserReturn<'a>,
}

impl<'a> AuthoredProgram<'a> {
    #[must_use]
    pub fn parse(arena: &'a Allocator, block: SourceBlock<'a>, source_type: SourceType) -> Self {
        let parsed = Parser::new(arena.as_oxc(), block.source(), source_type).parse();
        Self {
            block,
            source_type,
            parsed,
        }
    }

    #[must_use]
    pub const fn block(&self) -> SourceBlock<'a> {
        self.block
    }

    /// The requested profile survives the parser's resolution of Unambiguous inputs.
    #[must_use]
    pub const fn source_type(&self) -> SourceType {
        self.source_type
    }

    #[must_use]
    pub const fn parsed(&self) -> &ParserReturn<'a> {
        &self.parsed
    }
}

/// Parse each requested stock grammar once for this block, in first-request order.
/// An absent profile needs no parse. The callback owns its original Program for
/// the duration of the group so a final tool can consume it after read-only passes.
/// A failed reader stops the schedule before any later grammar is parsed.
pub fn visit_authored_profiles<E>(
    block: SourceBlock<'_>,
    profiles: &[Option<SourceType>],
    mut read: impl for<'p> FnMut(AuthoredProgram<'p>) -> Result<(), E>,
) -> Result<(), E> {
    for (i, &profile) in profiles.iter().enumerate() {
        if let Some(grammar) = profile
            && !profiles[..i].contains(&profile)
        {
            let arena = Allocator::default();
            read(AuthoredProgram::parse(&arena, block, grammar))?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use vize_l0::SourceRoot;

    #[test]
    fn one_original_per_requested_grammar_and_no_parse_after_refusal() {
        let source = "<script>const x: number = 1</script>";
        let root = SourceRoot::new(source).unwrap();
        let block = root.block(&source[8..27], 8).unwrap();
        let js = Some(SourceType::mjs());
        let ts = Some(SourceType::ts());
        let mut reads = alloc::vec::Vec::new();
        let result = visit_authored_profiles(
            block,
            &[None, js, js, ts, None, Some(SourceType::tsx())],
            |syntax| {
                assert!(core::ptr::eq(syntax.block().root_source(), source));
                assert!(core::ptr::eq(syntax.block().source(), block.source()));
                reads.push((syntax.source_type(), syntax.parsed().diagnostics.is_empty()));
                if Some(syntax.source_type()) == ts {
                    Err(7)
                } else {
                    Ok(())
                }
            },
        );
        assert_eq!(result, Err(7));
        assert_eq!(
            reads,
            [(SourceType::mjs(), false), (SourceType::ts(), true)]
        );
        visit_authored_profiles::<()>(block, &[None, None], |_| panic!("unexpected parse"))
            .unwrap();
    }
}
