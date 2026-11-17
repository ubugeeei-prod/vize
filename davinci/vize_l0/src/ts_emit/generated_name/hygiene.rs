//! Keep generated names apart from the identifiers an SFC authors.
//!
//! A projection binds the `$`-identifiers its SFC authors (script bindings,
//! template expressions and template scopes). A generated name that one of
//! them spells, or whose family it falls in, takes the first spelling with
//! trailing `$`s that none of them uses: `$args` becomes `$args$` beside an
//! authored `const $args`. Everything else keeps its constant spelling, so an
//! SFC without such a binding pays one flag check per name.

use std::borrow::Cow;
use std::cell::{Cell, RefCell};

use super::{ALL, GeneratedName};

/// Generated names an SFC authors and the `$`s each takes.
type Renamed = Vec<(&'static str, usize)>;
/// Interned renamed spellings: name, `$` count, spelling.
type Interned = Vec<(&'static str, usize, &'static str)>;

thread_local! {
    /// Projections are synchronous per thread, like canon's other scopes.
    static ACTIVE: Cell<bool> = const { Cell::new(false) };
    static FRESH: RefCell<Renamed> = const { RefCell::new(Vec::new()) };
}

/// Restores the outer projection's spellings on return and unwinding.
#[must_use = "the spellings revert when the scope drops"]
pub struct Scope {
    outer: Option<Renamed>,
}

impl Drop for Scope {
    fn drop(&mut self) {
        let outer = self.outer.take().unwrap_or_default();
        ACTIVE.with(|active| active.set(!outer.is_empty()));
        FRESH.with_borrow_mut(|fresh| *fresh = outer);
    }
}

/// Bind the authored identifiers of one projection. Only identifiers that
/// start with `$` can collide; others are ignored.
pub fn bind<'a>(authored: impl IntoIterator<Item = &'a str>) -> Scope {
    let authored: Vec<&str> = authored
        .into_iter()
        .filter(|ident| ident.starts_with('$'))
        .collect();
    let fresh: Renamed = if authored.is_empty() {
        Vec::new()
    } else {
        ALL.iter()
            .filter_map(|name| {
                let count = fresh_count(name.as_str(), &authored);
                (count > 0).then_some((name.as_str(), count))
            })
            .collect()
    };
    ACTIVE.with(|active| active.set(!fresh.is_empty()));
    let outer = FRESH.with_borrow_mut(|current| std::mem::replace(current, fresh));
    Scope { outer: Some(outer) }
}

/// Every `$`-identifier token of `text`, for [`bind`].
pub fn dollar_identifiers(text: &str) -> impl Iterator<Item = &str> {
    let bytes = text.as_bytes();
    let mut index = 0;
    core::iter::from_fn(move || {
        while index < bytes.len() {
            let start = memchr::memchr(b'$', bytes.get(index..)?)? + index;
            let continues_word = start
                .checked_sub(1)
                .and_then(|before| bytes.get(before))
                .is_some_and(|&byte| is_word(byte));
            let mut end = start + 1;
            while bytes.get(end).is_some_and(|&byte| is_word(byte)) {
                end += 1;
            }
            index = end;
            if !continues_word {
                return text.get(start..end);
            }
        }
        None
    })
}

fn is_word(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'$' || byte >= 0x80
}

fn collides(spelling: &str, ident: &str) -> bool {
    ident.strip_prefix(spelling).is_some_and(|tail| {
        tail.bytes()
            .next()
            .is_none_or(|byte| byte.is_ascii_digit() || byte == b'_' || byte == b'$')
    })
}

fn fresh_count(base: &str, authored: &[&str]) -> usize {
    let mut spelling = String::from(base);
    let mut count = 0;
    while authored.iter().any(|ident| collides(&spelling, ident)) {
        spelling.push('$');
        count += 1;
    }
    count
}

/// The number of `$`s the bound projection appends to `name`.
#[inline]
pub(super) fn fresh(name: &'static str) -> usize {
    if !ACTIVE.with(Cell::get) {
        return 0;
    }
    FRESH.with_borrow(|fresh| {
        fresh
            .iter()
            .find(|(candidate, _)| core::ptr::eq(*candidate, name) || *candidate == name)
            .map_or(0, |&(_, count)| count)
    })
}

/// Generated static `text` with every renamed name in its bound spelling.
/// Never pass authored text.
pub fn text(text: &str) -> Cow<'_, str> {
    if !ACTIVE.with(Cell::get) {
        return Cow::Borrowed(text);
    }
    FRESH.with_borrow(|fresh| {
        let mut out = String::new();
        let mut copied = 0;
        for ident in dollar_identifiers(text) {
            let Some(&(name, count)) = fresh.iter().find(|(name, _)| collides(name, ident)) else {
                continue;
            };
            let start = ident.as_ptr() as usize - text.as_ptr() as usize;
            out.push_str(text.get(copied..start).unwrap_or_default());
            out.push_str(name);
            (0..count).for_each(|_| out.push('$'));
            copied = start + name.len();
        }
        if copied == 0 {
            Cow::Borrowed(text)
        } else {
            out.push_str(text.get(copied..).unwrap_or_default());
            Cow::Owned(out)
        }
    })
}

/// Whether any name is renamed in the bound projection.
pub fn is_active() -> bool {
    ACTIVE.with(Cell::get)
}

/// `name` with `count` trailing `$`s, leaked once per spelling: the set is
/// bounded by the table and only renamed names reach it.
pub(super) fn intern(name: &'static str, count: usize) -> &'static str {
    use std::sync::{Mutex, OnceLock};
    static INTERNED: OnceLock<Mutex<Interned>> = OnceLock::new();
    let interned = INTERNED.get_or_init(Default::default);
    let mut interned = interned
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if let Some(&(_, _, spelled)) = interned
        .iter()
        .find(|(candidate, fresh, _)| *candidate == name && *fresh == count)
    {
        return spelled;
    }
    let mut spelled = String::from(name);
    (0..count).for_each(|_| spelled.push('$'));
    let spelled: &'static str = Box::leak(spelled.into_boxed_str());
    interned.push((name, count, spelled));
    spelled
}

/// The bound spelling of `name`, owned.
pub fn spelling(name: GeneratedName) -> crate::String {
    let mut spelling = crate::String::from(name.as_str());
    (0..name.fresh()).for_each(|_| spelling.push('$'));
    spelling
}
