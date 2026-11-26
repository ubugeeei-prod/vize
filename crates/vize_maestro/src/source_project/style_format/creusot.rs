//! Creusot proves the style formatter's header acceptance on its shipped
//! source (`header.rs`). This file is the library root of
//! `tests/formal/creusot/style-header` and is not part of `vize_maestro`'s
//! module tree. `header.rs` is included verbatim, as the product includes
//! it, over the two items it reads: an attribute's name and optional value,
//! and the ordered view of a style block's original attributes.
//!
//! `same_header` carries its contract (`accepts`, the Lean model's
//! acceptance) in the shipped source. The law below is the statement of
//! `accepted_iff` in tests/formal/style-format/Header.lean, and of the Kani
//! harness it replaces, which bounded each header at four attributes over
//! five names and four values. It holds for headers of every length and
//! every spelling. See docs/davinci/decisions/2026-11-17-creusot.md.

use creusot_std::{logic::FSet, prelude::*};

include!("header.rs");

/// The attribute fields the policy reads.
pub struct BlockAttr<'a> {
    pub name: &'a str,
    pub value: Option<&'a str>,
}

/// The ordered view of a style block's original attribute spellings.
pub struct StyleView<'v, 'a>(pub &'v [BlockAttr<'a>]);

impl<'v, 'a> View for StyleView<'v, 'a> {
    type ViewTy = Seq<BlockAttr<'a>>;

    #[logic(open)]
    fn view(self) -> Seq<BlockAttr<'a>> {
        pearlite! { self.0@ }
    }
}

impl<'v, 'a> StyleView<'v, 'a> {
    #[ensures(result@ == self@)]
    fn attrs(&self) -> &'v [BlockAttr<'a>] {
        self.0
    }
}

/// How many of the first `n` attributes of `s` are spelled `x`.
#[logic(open(crate))]
#[variant(n)]
pub fn occurrences(s: Seq<BlockAttr<'_>>, x: (Seq<char>, Option<Seq<char>>), n: Int) -> Int {
    pearlite! {
        if n <= 0 {
            0
        } else if spelling(s[n - 1]) == x {
            occurrences(s, x, n - 1) + 1
        } else {
            occurrences(s, x, n - 1)
        }
    }
}

/// The distinct spellings among the first `n` attributes of `s`.
#[logic(open(crate))]
#[variant(n)]
pub fn spellings(s: Seq<BlockAttr<'_>>, n: Int) -> FSet<(Seq<char>, Option<Seq<char>>)> {
    pearlite! {
        if n <= 0 { FSet::empty() } else { spellings(s, n - 1).insert(spelling(s[n - 1])) }
    }
}

/// No two of the first `n` attributes of `s` are spelled alike.
#[logic(open(crate))]
pub fn distinct(s: Seq<BlockAttr<'_>>, n: Int) -> bool {
    pearlite! { forall<i: Int, j: Int> 0 <= i && i < j && j < n ==> spelling(s[i]) != spelling(s[j]) }
}

/// A spelling occurs exactly when some attribute has it.
#[logic]
#[variant(n)]
#[requires(0 <= n && n <= s.len())]
#[ensures((occurrences(s, x, n) > 0) == exists<k: Int> 0 <= k && k < n && spelling(s[k]) == x)]
#[ensures(spellings(s, n).contains(x) == exists<k: Int> 0 <= k && k < n && spelling(s[k]) == x)]
#[ensures(occurrences(s, x, n) >= 0)]
pub fn a_spelling_occurs_where_an_attribute_has_it(
    s: Seq<BlockAttr<'_>>,
    x: (Seq<char>, Option<Seq<char>>),
    n: Int,
) {
    pearlite! { if n > 0 { a_spelling_occurs_where_an_attribute_has_it(s, x, n - 1) } }
}

/// The spellings of `n` attributes number at most `n`, and exactly `n`
/// only when no two are alike; a distinct header spells each once.
#[logic]
#[variant(n)]
#[requires(0 <= n && n <= s.len())]
#[ensures(spellings(s, n).len() <= n)]
#[ensures(spellings(s, n).len() == n == distinct(s, n))]
#[ensures(distinct(s, n) ==> forall<x: (Seq<char>, Option<Seq<char>>)> occurrences(s, x, n) <= 1)]
pub fn distinct_spellings_are_counted_once(s: Seq<BlockAttr<'_>>, n: Int) {
    pearlite! {
        if n > 0 {
            let _ = distinct_spellings_are_counted_once(s, n - 1);
            let _ = a_spelling_occurs_where_an_attribute_has_it(s, spelling(s[n - 1]), n - 1);
        }
    }
}

/// Distinct spellings that all occur in an equally long header exhaust it,
/// so each occurs there exactly once (pigeonhole).
#[logic]
#[requires(a.len() == b.len() && distinct(a, a.len()))]
#[requires(forall<i: Int> 0 <= i && i < a.len()
    ==> exists<k: Int> 0 <= k && k < b.len() && spelling(b[k]) == spelling(a[i]))]
#[ensures(forall<i: Int> 0 <= i && i < a.len() ==> occurrences(b, spelling(a[i]), b.len()) == 1)]
pub fn distinct_spellings_found_in_an_equal_header_occur_once(
    a: Seq<BlockAttr<'_>>,
    b: Seq<BlockAttr<'_>>,
) {
    pearlite! {
        {
            let _ = distinct_spellings_are_counted_once(a, a.len());
            let _ = distinct_spellings_are_counted_once(b, b.len());
            proof_assert!(forall<x: (Seq<char>, Option<Seq<char>>)> spellings(a, a.len()).contains(x)
                ==> spellings(b, b.len()).contains(x));
        }
    }
}

/// The Kani harness's and the Lean model's acceptance: unchanged, or
/// distinct plain flags, each spelled exactly once in a header of the same
/// length.
#[logic(open(crate))]
pub fn reorders_plain_flags(a: Seq<BlockAttr<'_>>, b: Seq<BlockAttr<'_>>) -> bool {
    pearlite! {
        a.len() == b.len()
            && (forall<i: Int> 0 <= i && i < a.len() ==> plain(a[i].name.deep_model()))
            && (forall<i: Int, j: Int> 0 <= i && i < a.len() && 0 <= j && j < a.len() && i != j
                ==> a[i].name.deep_model() != a[j].name.deep_model())
            && forall<i: Int> 0 <= i && i < a.len() ==> occurrences(b, spelling(a[i]), b.len()) == 1
    }
}

/// Unchanged headers, attribute by attribute.
#[logic(open(crate))]
pub fn unchanged(a: Seq<BlockAttr<'_>>, b: Seq<BlockAttr<'_>>) -> bool {
    pearlite! {
        a.len() == b.len() && forall<i: Int> 0 <= i && i < a.len() ==> spelling(a[i]) == spelling(b[i])
    }
}

/// Every attribute the scan accepts has a name no other attribute has.
#[logic]
#[requires(forall<i: Int> 0 <= i && i < a.len() ==> scanned(a, b, i))]
#[ensures(forall<i: Int, j: Int> 0 <= i && i < a.len() && 0 <= j && j < a.len() && i != j
    ==> a[i].name.deep_model() != a[j].name.deep_model())]
#[ensures(distinct(a, a.len()))]
pub fn scanned_names_are_distinct(a: Seq<BlockAttr<'_>>, b: Seq<BlockAttr<'_>>) {}

/// A scan that accepts reorders plain flags.
#[logic]
#[requires(a.len() == b.len())]
#[requires(forall<i: Int> #[trigger(a[i])] 0 <= i && i < a.len() ==> scanned(a, b, i))]
#[ensures(reorders_plain_flags(a, b))]
pub fn an_accepting_scan_reorders_plain_flags(a: Seq<BlockAttr<'_>>, b: Seq<BlockAttr<'_>>) {
    pearlite! {
        {
            let _ = scanned_names_are_distinct(a, b);
            proof_assert!(forall<i: Int> 0 <= i && i < a.len()
                ==> scanned(a, b, i) && plain(a[i].name.deep_model()));
            proof_assert!(forall<i: Int> 0 <= i && i < a.len()
                ==> scanned(a, b, i)
                    && exists<k: Int> 0 <= k && k < b.len() && spelling(b[k]) == spelling(a[i]));
            distinct_spellings_found_in_an_equal_header_occur_once(a, b)
        }
    }
}

/// Reordered plain flags pass the scan.
#[logic]
#[requires(reorders_plain_flags(a, b))]
#[ensures(forall<i: Int> 0 <= i && i < a.len() ==> scanned(a, b, i))]
pub fn reordered_plain_flags_pass_the_scan(a: Seq<BlockAttr<'_>>, b: Seq<BlockAttr<'_>>) {
    pearlite! {
        proof_assert!(forall<i: Int> 0 <= i && i < a.len()
            ==> a_spelling_occurs_where_an_attribute_has_it(b, spelling(a[i]), b.len()) == ())
    }
}

/// `accepted_iff`: a header is accepted exactly when it is unchanged, or
/// when its attributes are distinct plain flags and the new header holds
/// each of their spellings exactly once in a header of the same length.
#[ensures(result == (unchanged(a@, b@) || reorders_plain_flags(a@, b@)))]
pub fn accepted_headers_keep_order_or_permute_unique_plain_flags_with_their_values(
    a: &[BlockAttr<'_>],
    b: &[BlockAttr<'_>],
) -> bool {
    let accepted = same_header(StyleView(a), StyleView(b));
    proof_assert!(a@.len() == b@.len() && (forall<i: Int> 0 <= i && i < a@.len() ==> scanned(a@, b@, i))
        ==> an_accepting_scan_reorders_plain_flags(a@, b@) == ());
    proof_assert!(reorders_plain_flags(a@, b@) ==> reordered_plain_flags_pass_the_scan(a@, b@) == ());
    accepted
}
