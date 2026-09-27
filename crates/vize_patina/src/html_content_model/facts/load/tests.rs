//! Whole-state oracles for interned identities and the 256-element boundary.

use super::super::{Bits, Cond, WHATWG_TSV};
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

fn loaded() -> Facts {
    let mut defects = Vec::new();
    let facts = Facts::parse_reporting(WHATWG_TSV, &mut defects);
    assert_eq!(defects, Vec::<String>::new());
    assert_eq!(facts.names.len(), 146);
    facts
}

fn grow(expected: &mut State, extra: &[(Ns, &'static str, ElemId)]) {
    for &(ns, local, id) in extra {
        expected.names.push((ns, local));
        expected.ids.insert((ns, local), id);
    }
}

#[expect(clippy::disallowed_macros, reason = "indexed boundary fixture names")]
fn boundary_names() -> Vec<&'static str> {
    (146..256)
        .map(|id| {
            let name: &'static str = Box::leak(format!("fixture-{id:03}").into_boxed_str());
            name
        })
        .collect()
}

#[test]
fn existing_committed_table_hits_preserve_every_field() {
    let mut facts = loaded();
    let before = state(&facts);
    let universe: Vec<_> = facts.universe().collect();
    for (id, ns, local) in universe {
        assert_eq!(facts.intern(ns, local), Some(id));
    }
    assert_eq!(state(&facts), before);
}

#[test]
fn first_names_keep_empty_universe_namespace_and_case_order() {
    let mut facts = Facts::parse("");
    let mut expected = state(&facts);
    let extra = [
        (Ns::Html, "div", 0),
        (Ns::Svg, "div", 1),
        (Ns::MathMl, "div", 2),
        (Ns::Svg, "foreignObject", 3),
    ];
    grow(&mut expected, &extra);
    expected.cased.push((Ns::Svg, "foreignObject", 3));
    for (ns, local, id) in extra {
        assert_eq!(facts.intern(ns, local), Some(id));
    }
    assert_eq!(state(&facts), expected);
    assert_eq!(facts.id(Ns::Html, "DIV"), Some(0));
    assert_eq!(facts.id(Ns::Svg, "DIV"), Some(1));
    assert_eq!(facts.id(Ns::MathMl, "DIV"), Some(2));
    assert_eq!(facts.id(Ns::Svg, "foreignobject"), Some(3));
}

#[test]
fn nonempty_insertions_keep_exact_keys_ids_case_fallback_and_order() {
    let mut facts = loaded();
    let mut expected = state(&facts);
    let extra = [
        (Ns::Html, "fixture", 146),
        (Ns::Html, "FiXtUrE", 147),
        (Ns::Svg, "fixtureGradient", 148),
        (Ns::Svg, "fixturegradient", 149),
        (Ns::MathMl, "fixture", 150),
        (Ns::MathMl, "FiXtUrE", 151),
    ];
    grow(&mut expected, &extra);
    expected.cased.extend([
        (Ns::Html, "FiXtUrE", 147),
        (Ns::Svg, "fixtureGradient", 148),
        (Ns::MathMl, "FiXtUrE", 151),
    ]);
    for (ns, local, id) in extra {
        assert_eq!(facts.intern(ns, local), Some(id));
        assert_eq!(facts.intern(ns, local), Some(id));
    }
    assert_eq!(state(&facts), expected);
    let queries = [
        (Ns::Html, "fixture", Some(146)),
        (Ns::Html, "FiXtUrE", Some(147)),
        (Ns::Html, "FIXTURE", Some(146)),
        (Ns::Svg, "fixtureGradient", Some(148)),
        (Ns::Svg, "fixturegradient", Some(149)),
        (Ns::Svg, "FIXTUREGRADIENT", Some(148)),
        (Ns::MathMl, "fixture", Some(150)),
        (Ns::MathMl, "FiXtUrE", Some(151)),
        (Ns::MathMl, "FIXTURE", Some(150)),
        (Ns::Svg, "fixture", None),
        (Ns::Html, "fixtureGradient", None),
    ];
    for (ns, tag, id) in queries {
        assert_eq!(facts.id(ns, tag), id, "{ns:?} {tag}");
    }
    assert_eq!(state(&facts), expected);
}

#[test]
fn full_universe_rejects_vacancies_and_still_returns_all_occupied_ids() {
    let mut facts = loaded();
    let mut expected = state(&facts);
    let names = boundary_names();
    let extra: Vec<_> = names
        .iter()
        .enumerate()
        .map(|(index, &name)| (Ns::Html, name, 146 + index as ElemId))
        .collect();
    grow(&mut expected, &extra);
    for (ns, local, id) in extra {
        assert_eq!(facts.intern(ns, local), Some(id));
    }
    assert_eq!(facts.names.len(), 256);
    assert_eq!(state(&facts), expected);
    let universe: Vec<_> = facts.universe().collect();
    for (id, ns, local) in universe {
        assert_eq!(facts.intern(ns, local), Some(id));
    }
    for (ns, local) in [
        (Ns::Html, "fixture-overflow"),
        (Ns::Svg, "fixtureOverflow"),
        (Ns::MathMl, "fixtureOverflow"),
    ] {
        assert_eq!(facts.intern(ns, local), None);
        assert_eq!(facts.intern(ns, local), None);
        assert_eq!(facts.id(ns, local), None);
        assert_eq!(state(&facts), expected);
    }
    assert_eq!(facts.name(255), (Ns::Html, "fixture-255"));
}

#[test]
#[expect(clippy::disallowed_macros, reason = "complete overflow table fixture")]
fn loader_preserves_complete_overflow_defects_and_surviving_members() {
    let original = loaded();
    let mut expected = state(&original);
    let names = boundary_names();
    let extra: Vec<_> = names
        .iter()
        .enumerate()
        .map(|(index, &name)| (Ns::Html, name, 146 + index as ElemId))
        .collect();
    grow(&mut expected, &extra);
    let div = original
        .id(Ns::Html, "div")
        .expect("committed div identity");
    expected.children.insert(
        div,
        MemberState {
            always: Bits([0, 0, u64::MAX << 18, u64::MAX]),
            conditional: Vec::new(),
            text: false,
            anchor: "#entry-fixture",
        },
    );
    let tsv = format!(
        "{WHATWG_TSV}\nchildren\tdiv\t#entry-fixture\t{} fixture-overflow \
         svg:fixtureOverflow math:fixtureOverflow\n\
         children\tfixture-overflow-parent\t#entry-parent\tfixture-146\n",
        names.join(" ")
    );
    let mut defects = Vec::new();
    let facts = Facts::parse_reporting(Box::leak(tsv.into_boxed_str()), &mut defects);
    assert_eq!(
        defects,
        [
            String::from("element universe overflow at `fixture-overflow`"),
            String::from("element universe overflow at `svg:fixtureOverflow`"),
            String::from("element universe overflow at `math:fixtureOverflow`"),
            String::from("element universe overflow at `fixture-overflow-parent`"),
        ]
    );
    assert_eq!(state(&facts), expected);
}
