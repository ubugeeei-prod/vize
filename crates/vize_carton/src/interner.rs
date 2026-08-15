//! Arena-backed atoms for template names that repeat within one compile.
//!
//! Davinci P1-10 moved the AST's name-shaped fields off owned
//! [`CompactString`](compact_str::CompactString) onto `&'a str`. Three sources
//! can produce such a slice, and the choice per field is a real one:
//!
//! - **source slice** — the value is a verbatim run of the template text, so
//!   the node borrows it and nothing is allocated (most tags, attribute names,
//!   raw directive names, undecoded text);
//! - **atom** — the value is *computed* (normalized, camelized, synthesized)
//!   and the same computed value recurs across the file, so one arena copy is
//!   shared by every occurrence: this module;
//! - **arena copy** — the value is computed and effectively unique
//!   (entity-decoded text, concatenations), so [`Allocator::alloc_str`] copies
//!   it once and no bookkeeping is worth paying for.
//!
//! [`Interner`] is only worth reaching for in the middle case. Its fast path
//! costs nothing at all: names the compiler already knows — HTML/SVG/MathML
//! tags, directive names, the well-known attribute set — resolve to `'static`
//! literals through a compile-time perfect hash, so a `<div>`-heavy template
//! interns its tags without touching the arena or the hash set.
//!
//! [`Allocator::alloc_str`]: crate::Allocator::alloc_str

use phf::{Set as PhfSet, phf_set};

use crate::{Allocator, FxHashSet};

/// Template names the compiler ships knowledge of, as `'static` literals.
///
/// Membership here is a performance statement, not a semantic one: interning a
/// name that is absent simply takes the arena path. The set covers the names
/// that dominate real templates — every HTML/SVG/MathML tag the DOM tables
/// already list, every Vue directive name, and the attributes the transforms
/// special-case.
static WELL_KNOWN: PhfSet<&'static str> = phf_set! {
    // Structural / flow HTML tags (the head of the frequency distribution).
    "a", "abbr", "address", "area", "article", "aside", "audio",
    "b", "base", "bdi", "bdo", "blockquote", "body", "br", "button",
    "canvas", "caption", "cite", "code", "col", "colgroup",
    "data", "datalist", "dd", "del", "details", "dfn", "dialog", "div", "dl", "dt",
    "em", "embed",
    "fieldset", "figcaption", "figure", "footer", "form",
    "h1", "h2", "h3", "h4", "h5", "h6", "head", "header", "hgroup", "hr", "html",
    "i", "iframe", "img", "input", "ins",
    "kbd",
    "label", "legend", "li", "link",
    "main", "map", "mark", "menu", "meta", "meter",
    "nav", "noscript",
    "object", "ol", "optgroup", "option", "output",
    "p", "param", "picture", "pre", "progress",
    "q",
    "rp", "rt", "ruby",
    "s", "samp", "script", "search", "section", "select", "small", "source",
    "span", "strong", "style", "sub", "summary", "sup",
    "table", "tbody", "td", "template", "textarea", "tfoot", "th", "thead",
    "time", "title", "tr", "track",
    "u", "ul",
    "var", "video",
    "wbr",
    // SVG / MathML tags that appear inside templates.
    "circle", "clipPath", "defs", "ellipse", "foreignObject", "g",
    "line", "linearGradient", "marker", "mask", "path", "pattern",
    "polygon", "polyline", "radialGradient", "rect", "stop", "svg",
    "symbol", "text", "textPath", "tspan", "use", "view",
    "math", "mi", "mn", "mo", "mrow", "ms", "msqrt", "mtext",
    // Vue built-in components.
    "component", "keep-alive", "KeepAlive", "slot", "suspense", "Suspense",
    "teleport", "Teleport", "transition", "Transition",
    "transition-group", "TransitionGroup",
    // Directive names, as normalized by the parser. `html`, `pre` and `text`
    // are directive names too, but already listed above as tags.
    "bind", "cloak", "else", "else-if", "for", "if", "is", "memo",
    "model", "on", "once", "show",
    // Well-known attributes the transforms special-case (`style` and `title`
    // are listed above as tags).
    "class", "id", "key", "name", "ref", "src", "type", "value",
    "checked", "disabled", "href", "placeholder", "selected",
};

/// Returns the `'static` atom equal to `name`, when the compiler ships it.
///
/// This is the interner's allocation-free fast path, exposed on its own for
/// call sites that have no interner to hand.
#[inline]
#[must_use]
pub fn well_known_atom(name: &str) -> Option<&'static str> {
    WELL_KNOWN.get_key(name).copied()
}

/// Per-compile name interner: one arena copy per distinct computed name.
///
/// Reach for this when the value handed to [`Interner::intern`] is *computed*
/// and recurs — a camelized prop name, a synthesized asset name — so the
/// second and later occurrences cost a hash lookup instead of a copy. A value
/// that is already a slice of the template needs no interner at all, and a
/// value that occurs once is cheaper through [`Allocator::alloc_str`].
///
/// The returned atoms live in `allocator` and therefore expire with it: they
/// are per-compile ephemera under the arena/cache contract and must be
/// converted to an owned form before crossing a compile boundary.
///
/// [`Allocator::alloc_str`]: crate::Allocator::alloc_str
///
/// # Example
///
/// ```
/// use vize_carton::{Allocator, interner::Interner};
///
/// let allocator = Allocator::default();
/// let mut interner = Interner::new(&allocator);
///
/// // A well-known name resolves to a `'static` literal: nothing is allocated.
/// assert_eq!(interner.intern("div"), "div");
/// assert_eq!(interner.len(), 0);
///
/// // A computed name is copied once and shared by every later occurrence.
/// let first = interner.intern("fooBar");
/// let second = interner.intern("fooBar");
/// assert_eq!(first, "fooBar");
/// assert!(std::ptr::eq(first, second));
/// assert_eq!(interner.len(), 1);
/// ```
pub struct Interner<'a> {
    allocator: &'a Allocator,
    seen: FxHashSet<&'a str>,
}

impl<'a> Interner<'a> {
    /// Creates an interner over `allocator`.
    #[inline]
    #[must_use]
    pub fn new(allocator: &'a Allocator) -> Self {
        Self {
            allocator,
            seen: FxHashSet::default(),
        }
    }

    /// Returns an arena-lifetime atom equal to `name`.
    ///
    /// Repeated calls with equal names return the same pointer, so the copy is
    /// paid once per distinct computed name per compile.
    #[inline]
    pub fn intern(&mut self, name: &str) -> &'a str {
        if let Some(atom) = well_known_atom(name) {
            return atom;
        }
        if let Some(existing) = self.seen.get(name) {
            return existing;
        }
        let atom: &'a str = self.allocator.alloc_str(name);
        self.seen.insert(atom);
        atom
    }

    /// Number of distinct names this interner has copied into the arena.
    ///
    /// Well-known names are not counted: they never reach the arena.
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.seen.len()
    }

    /// Whether this interner has copied anything into the arena.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.seen.is_empty()
    }
}

impl std::fmt::Debug for Interner<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Interner")
            .field("interned", &self.seen.len())
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::{Interner, well_known_atom};
    use crate::Allocator;

    #[test]
    fn well_known_names_resolve_to_static_literals() {
        assert_eq!(well_known_atom("div"), Some("div"));
        assert_eq!(well_known_atom("bind"), Some("bind"));
        assert_eq!(well_known_atom("class"), Some("class"));
        assert_eq!(well_known_atom("MyComponent"), None);
    }

    #[test]
    fn well_known_names_never_touch_the_arena() {
        let allocator = Allocator::new();
        let mut interner = Interner::new(&allocator);
        assert_eq!(interner.intern("div"), "div");
        assert_eq!(interner.intern("span"), "span");
        assert!(interner.is_empty());
        assert_eq!(allocator.allocated_bytes(), 0);
    }

    #[test]
    fn computed_names_are_copied_once_and_shared() {
        let allocator = Allocator::new();
        let mut interner = Interner::new(&allocator);
        let first = interner.intern("onCustomEvent");
        let second = interner.intern("onCustomEvent");
        assert_eq!(first, "onCustomEvent");
        assert!(std::ptr::eq(first, second));
        assert_eq!(interner.len(), 1);
    }

    #[test]
    fn distinct_names_get_distinct_atoms() {
        let allocator = Allocator::new();
        let mut interner = Interner::new(&allocator);
        let a = interner.intern("alpha");
        let b = interner.intern("beta");
        assert_eq!((a, b), ("alpha", "beta"));
        assert_eq!(interner.len(), 2);
    }

    #[test]
    fn atoms_outlive_the_interner() {
        let allocator = Allocator::new();
        let atom = {
            let mut interner = Interner::new(&allocator);
            interner.intern("survives")
        };
        assert_eq!(atom, "survives");
    }
}
