// Shared with the Creusot proof root (`creusot.rs`) so it proves this exact
// policy. See tests/formal/style-format/Header.lean for the law. The root
// brings the Creusot prelude into scope.
const LANG: &str = "lang";
const SCOPED: &str = "scoped";
const MODULE: &str = "module";

/// The formatted block keeps the original header: the same attribute
/// spellings in the same order, or a reordering of plain SFC flags.
#[cfg_attr(creusot, ensures(result == accepts(before@, after@)))]
fn same_header(before: StyleView<'_, '_>, after: StyleView<'_, '_>) -> bool {
    let a = before.attrs();
    let b = after.attrs();
    if a.len() != b.len() {
        return false;
    }
    if identical(a, b) {
        return true;
    }
    // The stock formatter can order plain SFC header flags. Duplicates and
    // dynamic or unfamiliar attributes retain their original ordering contract.
    let mut prior = 0;
    #[cfg_attr(creusot, invariant(prior@ <= a@.len()))]
    #[cfg_attr(creusot, invariant(forall<i: Int> 0 <= i && i < prior@ ==> scanned(a@, b@, i)))]
    #[cfg_attr(creusot, variant(a@.len() - prior@))]
    while let Some(attr) = a.get(prior) {
        let plain = attr.name == LANG || attr.name == SCOPED || attr.name == MODULE;
        let (earlier, _) = a.split_at(prior);
        if !plain || names(earlier, attr.name) || !spells(b, attr.name, attr.value) {
            return false;
        }
        #[cfg(creusot)]
        proof_assert!(*attr == a@[prior@] && earlier@ == a@.subsequence(0, prior@));
        #[cfg(creusot)]
        proof_assert!(scanned(a@, b@, prior@));
        prior += 1;
    }
    true
}

/// Both headers spell every attribute alike, in order.
#[cfg_attr(creusot, requires(a@.len() == b@.len()))]
#[cfg_attr(creusot, ensures(result == forall<i: Int> 0 <= i && i < a@.len() ==> spelling(a@[i]) == spelling(b@[i])))]
fn identical(a: &[BlockAttr<'_>], b: &[BlockAttr<'_>]) -> bool {
    let mut at = 0;
    #[cfg_attr(creusot, invariant(at@ <= a@.len()))]
    #[cfg_attr(creusot, invariant(forall<i: Int> 0 <= i && i < at@ ==> spelling(a@[i]) == spelling(b@[i])))]
    #[cfg_attr(creusot, variant(a@.len() - at@))]
    while let (Some(x), Some(y)) = (a.get(at), b.get(at)) {
        if x.name != y.name || x.value != y.value {
            return false;
        }
        at += 1;
    }
    true
}

/// Some attribute in `attrs` has this name.
#[cfg_attr(creusot, ensures(result == exists<i: Int> 0 <= i && i < attrs@.len()
    && attrs@[i].name.deep_model() == name.deep_model()))]
fn names(attrs: &[BlockAttr<'_>], name: &str) -> bool {
    let mut at = 0;
    #[cfg_attr(creusot, invariant(at@ <= attrs@.len()))]
    #[cfg_attr(creusot, invariant(forall<i: Int> 0 <= i && i < at@ ==> attrs@[i].name.deep_model() != name.deep_model()))]
    #[cfg_attr(creusot, variant(attrs@.len() - at@))]
    while let Some(other) = attrs.get(at) {
        if other.name == name {
            return true;
        }
        at += 1;
    }
    false
}

/// Some attribute in `attrs` is spelled exactly so.
#[cfg_attr(creusot, ensures(result == exists<i: Int> 0 <= i && i < attrs@.len()
    && spelling(attrs@[i]) == (name.deep_model(), value.deep_model())))]
fn spells(attrs: &[BlockAttr<'_>], name: &str, value: Option<&str>) -> bool {
    let mut at = 0;
    #[cfg_attr(creusot, invariant(at@ <= attrs@.len()))]
    #[cfg_attr(creusot, invariant(forall<i: Int> 0 <= i && i < at@
        ==> spelling(attrs@[i]) != (name.deep_model(), value.deep_model())))]
    #[cfg_attr(creusot, variant(attrs@.len() - at@))]
    while let Some(other) = attrs.get(at) {
        if other.name == name && other.value == value {
            return true;
        }
        at += 1;
    }
    false
}

/// An attribute's spelling: its name and its optional value.
#[cfg(creusot)]
#[logic(open(crate))]
fn spelling(attr: BlockAttr<'_>) -> (Seq<char>, Option<Seq<char>>) {
    pearlite! { (attr.name.deep_model(), attr.value.deep_model()) }
}

/// `lang`, `scoped` and `module` are the flags the formatter may reorder.
#[cfg(creusot)]
#[logic(open(crate))]
fn plain(name: Seq<char>) -> bool {
    pearlite! { name == LANG.deep_model() || name == SCOPED.deep_model() || name == MODULE.deep_model() }
}

/// Attribute `i` of `a` is a plain flag, its name is new, and `b` spells
/// it (`scan` in the Lean model).
#[cfg(creusot)]
#[logic(open(crate))]
fn scanned(a: Seq<BlockAttr<'_>>, b: Seq<BlockAttr<'_>>, i: Int) -> bool {
    pearlite! {
        plain(a[i].name.deep_model())
            && (forall<j: Int> 0 <= j && j < i ==> a[j].name.deep_model() != a[i].name.deep_model())
            && exists<k: Int> 0 <= k && k < b.len() && spelling(b[k]) == spelling(a[i])
    }
}

/// `accepts` in the Lean model.
#[cfg(creusot)]
#[logic(open(crate))]
fn accepts(a: Seq<BlockAttr<'_>>, b: Seq<BlockAttr<'_>>) -> bool {
    pearlite! {
        a.len() == b.len()
            && ((forall<i: Int> 0 <= i && i < a.len() ==> spelling(a[i]) == spelling(b[i]))
                || forall<i: Int> 0 <= i && i < a.len() ==> scanned(a, b, i))
    }
}
