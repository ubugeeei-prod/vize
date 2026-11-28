//! Musea's target inference for an Art file without an explicit target.
//!
//! The target is the one imported binding every variant renders as a tag. If
//! no binding is shared by every variant, it is the one imported binding any
//! variant renders. Several candidates are ambiguous and need
//! `defineArt(Target)`. Musea's parser and the Musea lint rules both decide
//! through [`infer_art_target`], so a file the gallery renders is never
//! reported as missing its target.

use vize_carton::{CompactString, FxHashMap};

/// The outcome of [`infer_art_target`].
#[derive(Debug, PartialEq, Eq)]
pub enum ArtTarget<'p> {
    /// No imported binding is rendered by a variant, or there are no variants.
    None,
    /// The one target: its local binding name and import source.
    One {
        /// Local binding name, which is also the rendered tag.
        name: &'p str,
        /// Import source of the binding.
        source: &'p str,
    },
    /// Several bindings qualify; their names, sorted.
    Ambiguous(Vec<&'p str>),
}

/// Infer the target of an Art file from its script-setup `import_sources`
/// (local name to source) and its `variants`, whose rendered markup
/// `template` returns.
pub fn infer_art_target<'p, 'v, V>(
    import_sources: &'p FxHashMap<CompactString, CompactString>,
    variants: &'v [V],
    template: impl Fn(&'v V) -> &'v str,
) -> ArtTarget<'p> {
    if variants.is_empty() {
        return ArtTarget::None;
    }
    let rendered = |variant: &'v V, name: &CompactString| {
        template_uses_component(template(variant), name.as_str())
    };
    let shared = import_sources
        .iter()
        .filter(|(name, _)| variants.iter().all(|variant| rendered(variant, name)))
        .collect::<Vec<_>>();
    let candidates = if shared.is_empty() {
        import_sources
            .iter()
            .filter(|(name, _)| variants.iter().any(|variant| rendered(variant, name)))
            .collect::<Vec<_>>()
    } else {
        shared
    };
    match candidates.as_slice() {
        [] => ArtTarget::None,
        [(name, source)] => ArtTarget::One {
            name: name.as_str(),
            source: source.as_str(),
        },
        _ => {
            let mut names = candidates
                .iter()
                .map(|(name, _)| name.as_str())
                .collect::<Vec<_>>();
            names.sort_unstable();
            ArtTarget::Ambiguous(names)
        }
    }
}

/// Whether `template` opens a `<name>` tag.
pub fn template_uses_component(template: &str, name: &str) -> bool {
    let mut cursor = 0;
    while let Some(relative) = template.get(cursor..).and_then(|rest| rest.find('<')) {
        let start = cursor + relative + 1;
        let Some(rest) = template.get(start..) else {
            return false;
        };
        if rest.starts_with(name)
            && rest
                .as_bytes()
                .get(name.len())
                .is_some_and(|byte| byte.is_ascii_whitespace() || matches!(byte, b'/' | b'>'))
        {
            return true;
        }
        cursor = start;
    }
    false
}
