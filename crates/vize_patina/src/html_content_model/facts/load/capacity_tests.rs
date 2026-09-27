//! Complete logical-state oracles for the first parsed-row capacity hint.

use super::super::{Attr, Bits, Cond};
use super::{ElemId, Facts, FxHashMap, Members, Ns, String};

#[derive(Clone, Debug, PartialEq, Eq)]
struct MemberState {
    always: Bits,
    conditional: Vec<(ElemId, Cond)>,
    text: bool,
    anchor: &'static str,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct State {
    names: Vec<(Ns, &'static str)>,
    ids: FxHashMap<(Ns, &'static str), ElemId>,
    cased: Vec<(Ns, &'static str, ElemId)>,
    rows: Vec<MemberState>,
    children: FxHashMap<ElemId, MemberState>,
    empty: MemberState,
}

fn member_state(row: &Members) -> MemberState {
    MemberState {
        always: row.always,
        conditional: row.conditional.iter().copied().collect(),
        text: row.text,
        anchor: row.anchor,
    }
}

fn state(facts: &Facts) -> State {
    State {
        names: facts.names.clone(),
        ids: facts.ids.clone(),
        cased: facts.cased.clone(),
        rows: facts.rows.iter().map(member_state).collect(),
        children: facts
            .children
            .iter()
            .map(|(&id, row)| (id, member_state(row)))
            .collect(),
        empty: member_state(&facts.empty),
    }
}

fn blank() -> MemberState {
    MemberState {
        always: Bits([0; 4]),
        conditional: Vec::new(),
        text: false,
        anchor: "",
    }
}

fn expected(
    names: &[(Ns, &'static str)],
    cased: &[(Ns, &'static str, ElemId)],
    children: &[(ElemId, MemberState)],
) -> State {
    State {
        names: names.to_vec(),
        ids: names
            .iter()
            .enumerate()
            .map(|(id, &name)| (name, id as ElemId))
            .collect(),
        cased: cased.to_vec(),
        rows: vec![blank(); 27],
        children: children.iter().cloned().collect(),
        empty: blank(),
    }
}

fn missing_rows() -> Vec<String> {
    [
        "missing row `set special`",
        "missing row `set scope`",
        "missing row `set button-scope`",
        "missing row `set marker`",
        "missing row `set implied-end`",
        "missing row `set closes-p`",
        "missing row `set heading`",
        "missing row `set list-item-loop-transparent`",
        "missing row `set table-part`",
        "missing row `set document-part`",
        "missing row `set raw-text`",
        "missing row `set scripting-dependent`",
        "missing row `set void`",
        "missing row `set foreign-breakout`",
        "missing row `set mathml-text-integration`",
        "missing row `set html-integration`",
        "missing row `set table-children`",
        "missing row `set table-wrapped`",
        "missing row `set section-children`",
        "missing row `set section-wrapped`",
        "missing row `set row-children`",
        "missing row `set colgroup-children`",
        "missing row `set transparent`",
        "missing row `category phrasing`",
        "missing row `category heading`",
        "missing row `category interactive`",
        "missing row `category script-supporting`",
    ]
    .into_iter()
    .map(String::from)
    .collect()
}

#[test]
fn empty_and_malformed_prefixes_preserve_complete_state_and_defects() {
    for (tsv, prefix) in [
        ("", Vec::new()),
        ("\n# only a comment\n", Vec::new()),
        (
            "malformed\nchildren\tbroken\t#anchor\n",
            vec![
                String::from("malformed row `malformed`"),
                String::from("malformed row `children\tbroken\t#anchor`"),
            ],
        ),
    ] {
        let mut defects = Vec::new();
        let facts = Facts::parse_reporting(tsv, &mut defects);
        let mut expected_defects = prefix;
        expected_defects.extend(missing_rows());
        assert_eq!(defects, expected_defects);
        assert_eq!(state(&facts), expected(&[], &[], &[]));
    }
    let mut defects = Vec::new();
    let facts = Facts::parse_reporting(
        "malformed\nchildren\tfixture\t#first\tdiv svg:foreignObject math:mi\n",
        &mut defects,
    );
    let mut expected_defects = vec![String::from("malformed row `malformed`")];
    expected_defects.extend(missing_rows());
    assert_eq!(defects, expected_defects);
    assert_eq!(
        state(&facts),
        expected(
            &[
                (Ns::Html, "div"),
                (Ns::Svg, "foreignObject"),
                (Ns::MathMl, "mi"),
                (Ns::Html, "fixture"),
            ],
            &[(Ns::Svg, "foreignObject", 1)],
            &[(
                3,
                MemberState {
                    always: Bits([7, 0, 0, 0]),
                    anchor: "#first",
                    ..blank()
                },
            )],
        )
    );
}

#[test]
fn zero_direct_members_keep_unknown_row_and_reference_defect_order() {
    let mut defects = Vec::new();
    let facts = Facts::parse_reporting(
        "future\tignored\t#zero\t#text @later\nchildren\tfixture\t#later\tdiv\n",
        &mut defects,
    );
    let mut expected_defects = vec![
        String::from("`@later` used before its row"),
        String::from("unknown row `future ignored`"),
    ];
    expected_defects.extend(missing_rows());
    assert_eq!(defects, expected_defects);
    assert_eq!(
        state(&facts),
        expected(
            &[(Ns::Html, "div"), (Ns::Html, "fixture")],
            &[],
            &[(
                1,
                MemberState {
                    always: Bits([1, 0, 0, 0]),
                    anchor: "#later",
                    ..blank()
                },
            )],
        )
    );
}

#[test]
fn duplicate_and_invalid_tokens_before_row_validation_preserve_all_state() {
    let mut defects = Vec::new();
    let facts = Facts::parse_reporting(
        "future\tignored\t#first\tdiv div svg:foreignObject math:mi invalid?unknown #text @later\n\
         children\tfixture\t#child\tdiv svg:foreignObject?href math:mi #text\n",
        &mut defects,
    );
    let mut expected_defects = vec![
        String::from("unknown condition `unknown`"),
        String::from("`@later` used before its row"),
        String::from("unknown row `future ignored`"),
    ];
    expected_defects.extend(missing_rows());
    assert_eq!(defects, expected_defects);
    assert_eq!(
        state(&facts),
        expected(
            &[
                (Ns::Html, "div"),
                (Ns::Svg, "foreignObject"),
                (Ns::MathMl, "mi"),
                (Ns::Html, "fixture"),
            ],
            &[(Ns::Svg, "foreignObject", 1)],
            &[(
                3,
                MemberState {
                    always: Bits([5, 0, 0, 0]),
                    conditional: vec![(1, Cond::Has(Attr::Href))],
                    text: true,
                    anchor: "#child",
                },
            )],
        )
    );
}

#[test]
#[expect(
    clippy::disallowed_macros,
    reason = "complete first-row overflow fixture"
)]
fn oversized_first_row_keeps_exact_256_id_limit_and_complete_defects() {
    let names: Vec<&'static str> = (0..256)
        .map(|id| {
            let name: &'static str = Box::leak(format!("fixture-{id:03}").into_boxed_str());
            name
        })
        .collect();
    let table = format!(
        "future\tignored\t#full\t{} svg:fixtureOverflow math:fixtureOverflow\n\
         children\tfixture-overflow-parent\t#parent\tfixture-255\n",
        names.join(" ")
    );
    let mut defects = Vec::new();
    let mut facts = Facts::parse_reporting(Box::leak(table.into_boxed_str()), &mut defects);
    let mut expected_defects = vec![
        String::from("element universe overflow at `svg:fixtureOverflow`"),
        String::from("element universe overflow at `math:fixtureOverflow`"),
        String::from("unknown row `future ignored`"),
        String::from("element universe overflow at `fixture-overflow-parent`"),
    ];
    expected_defects.extend(missing_rows());
    assert_eq!(defects, expected_defects);
    let identities: Vec<_> = names.iter().map(|&name| (Ns::Html, name)).collect();
    let expected = expected(&identities, &[], &[]);
    assert_eq!(state(&facts), expected);
    for (id, &name) in names.iter().enumerate() {
        assert_eq!(facts.intern(Ns::Html, name), Some(id as ElemId));
    }
    assert_eq!(facts.intern(Ns::Svg, "fixtureOverflow"), None);
    assert_eq!(facts.intern(Ns::MathMl, "fixtureOverflow"), None);
    assert_eq!(state(&facts), expected);
}
