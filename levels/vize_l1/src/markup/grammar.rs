//! Markup grammars. Vue is the only grammar today; Svelte, Angular and
//! others join as further implementors without touching the shared lexer.

use super::directive::{DirectiveSyntax, VueDirectives};

/// A markup grammar: the syntax family a template is written in.
pub trait MarkupGrammar {
    /// Stable identifier used in dumps and diagnostics.
    const NAME: &'static str;
    /// The dialect syntax hook that decomposes directive attribute names.
    type Directives: DirectiveSyntax;
}

/// Vue template syntax (every Vue dialect; differences come from the dialect).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Vue;

impl MarkupGrammar for Vue {
    const NAME: &'static str = "vue";
    type Directives = VueDirectives;
}
