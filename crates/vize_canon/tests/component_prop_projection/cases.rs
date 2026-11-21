#[derive(Clone, Copy)]
pub(super) struct Case {
    pub(super) id: &'static str,
    declarations: &'static str,
    root: &'static str,
    pub(super) codes: &'static [u32],
}

pub(super) const WIDGET: &str =
    "declare function Widget(p: { fooBar: string; enabled?: boolean }): unknown;";
pub(super) const CASES: &[Case] = &[
    Case {
        id: "alias",
        declarations: WIDGET,
        root: "<Widget foo-bar='雪😀' enabled/>",
        codes: &[],
    },
    Case {
        id: "missing",
        declarations: WIDGET,
        root: "<Widget/>",
        codes: &[2345],
    },
    Case {
        id: "wrong_type",
        declarations: WIDGET,
        root: "<Widget foo-bar={12}/>",
        codes: &[2322],
    },
    Case {
        id: "extra",
        declarations: WIDGET,
        root: "<Widget foo-bar='ok' non-existent/>",
        codes: &[2353],
    },
    Case {
        id: "value_read",
        declarations: WIDGET,
        root: "<Widget foo-bar={absent}/>",
        codes: &[2304],
    },
    Case {
        id: "generic",
        declarations: "declare function Pair<T>(p: { value: T; use: (x: T) => void }): unknown;",
        root: "<Pair value={12} use={x => x.toFixed()}/>",
        codes: &[],
    },
    Case {
        id: "generic_inference",
        declarations: "declare function Pair<T>(p: { value: T; use: (x: T) => void }): unknown;",
        root: "<Pair value={12} use={x => x.toUpperCase()}/>",
        codes: &[2339],
    },
    Case {
        id: "member",
        declarations: "declare const UI: { Widget: (p: { text: string }) => unknown };",
        root: "<UI.Widget text={false}/>",
        codes: &[2322],
    },
    Case {
        id: "reserved",
        declarations: WIDGET,
        root: "<Widget key={missingKey} ref={missingRef} foo-bar='ok'/>",
        codes: &[2304, 2304],
    },
    Case {
        id: "spread",
        declarations: "declare function Widget(p: { fooBar: string }): unknown; const props = { 'foo-bar': 'ok', key: 3 };",
        root: "<Widget {...props}/>",
        codes: &[],
    },
    Case {
        id: "spread_before",
        declarations: "declare function Widget(p: { fooBar: string }): unknown; const props = { 'foo-bar': 3 };",
        root: "<Widget {...props} foo-bar='ok'/>",
        codes: &[],
    },
    Case {
        id: "data_aria",
        declarations: "declare function Widget(p: { 'data-雪': string; 'aria-label': string }): unknown;",
        root: "<Widget data-雪='é' aria-label='abc'/>",
        codes: &[],
    },
    Case {
        id: "unicode",
        declarations: "declare function Widget(p: { aSS: string }): unknown;",
        root: "<Widget a-ß={12}/>",
        codes: &[2322],
    },
    Case {
        id: "nested",
        declarations: WIDGET,
        root: "<div><Widget foo-bar='ok'/></div>",
        codes: &[],
    },
    Case {
        id: "missing_long_tag",
        declarations: "declare function VeryLongComponentOwner(p: { x: boolean; y: number }): unknown;",
        root: "<VeryLongComponentOwner x/>",
        codes: &[2345],
    },
    Case {
        id: "static_escape",
        declarations: "declare function Widget(p: { text: 'a\"b' }): unknown;",
        root: "<Widget text='a\"b'/>",
        codes: &[],
    },
    Case {
        id: "spread_overwrite",
        declarations: "declare function Widget(p: { fooBar: string }): unknown; const props = { fooBar: 3 };",
        root: "<Widget foo-bar='ok' {...props}/>",
        codes: &[2322, 2783],
    },
    Case {
        id: "constructor",
        declarations: "declare class Widget { $props: { count: number }; }",
        root: "<Widget count='x'/>",
        codes: &[2322],
    },
    Case {
        id: "conditional",
        declarations: "declare const when: boolean; declare function Widget(p: { ok: boolean }): unknown;",
        root: "<div>{when ? <Widget ok/> : <Widget ok={false}/>}</div>",
        codes: &[],
    },
    Case {
        id: "loop",
        declarations: "declare const values: number[]; declare function Widget(p: { count: number }): unknown;",
        root: "<div>{values.map(n => <Widget count={n}/>)}</div>",
        codes: &[],
    },
];

pub(super) fn source(case: Case) -> String {
    format!(
        "/* 😀 */ {}\nexport function View() {{ return {}; }}",
        case.declarations, case.root
    )
}
