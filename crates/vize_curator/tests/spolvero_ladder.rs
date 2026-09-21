//! TS-52 for the Spolvero stage ladder (C-2/C-3/C-5): `ladder_pages` feeds
//! every rung the Davinci pipeline has today - S1, the S2 lowering, the
//! transform plan's walks, one page per executed transform pass, and the S3
//! graph / partition / value pages - from the real stages. The feed validates against the committed schema, every
//! page is pinned by exact equality, and the S2/S3 spans index the S1 page
//! (the provenance property the playground's source highlighting relies on).

use std::path::Path;

use davinci_test_support::schema as schema_check;
use vize_curator::inspector::{SpolveroPage, ladder_pages, spolvero_value};

const TEMPLATE: &str = "\n  <div :class=\"cls\">{{ msg }}</div>\n  <Comp v-model=\"x\"><template #a>hi</template></Comp>\n";

const S2_PAGE: &str = r#"[disegno]
ops=8

[disegno.ops]
ui.element div @3:36
  ui.bind name="class" value=js("cls" @16:19) @8:20
  ui.interpolation js("msg" @24:27) @21:30
ui.component Comp @39:90
  ui.model read=js("x" @54:55) write=js("x" @54:55) @45:56
    attr element-kind="component" @45:56
  ui.element template @57:83
    ui.slot-content name="a" @67:69
    ui.text "hi" @70:72

"#;

/// The executed transform plan: the two mandatory barriers own a walk each,
/// and the two optional analyses share the third.
const S2_PLAN_PAGE: &str = "[fusion-plan-folio]
stage=s2
walks=3

[fusion-plan-folio.passes]
walk=0 pass=v-slot kind=mandatory-lowering fusability=barrier
walk=1 pass=v-model kind=mandatory-diagnostic fusability=barrier
walk=2 pass=hoist-static kind=optional fusability=fusable
walk=2 pass=template-complexity kind=optional fusability=fusable

";

const S2_PROVENANCE_PAGE: &str = r##"[s2-provenance-folio]

[s2-provenance-folio.records]
rule=condense.drop-whitespace node=- before="\n  " after="" @0:3
rule=lower.element node=0 before="<div :class=\"cls\">" after="ui.element div" @3:36
rule=lower.bind node=1 before=":class=\"cls\"" after="ui.bind \"class\"" @8:20
rule=lower.interpolation node=2 before=" msg " after="ui.interpolation js" @21:30
rule=condense.drop-whitespace node=- before="\n  " after="" @36:39
rule=lower.component node=3 before="<Comp v-model=\"x\">" after="ui.component Comp" @39:90
rule=lower.model node=4 before="v-model=\"x\"" after="ui.model" @45:56
rule=lower.element node=5 before="<template #a>" after="ui.element template" @57:83
rule=lower.slot-content node=6 before="#a" after="ui.slot-content \"a\"" @67:69
rule=lower.text node=7 before="hi" after="ui.text" @70:72
rule=condense.drop-whitespace node=- before="\n" after="" @90:91
rule=pass.v-slot.group node=3 before="Comp" after="groups \"a\"" @39:90
rule=pass.hoist-static.fact node=0 before="ui.element" after="level=not-static props=false nested=true native=true" @3:36
rule=pass.hoist-static.fact node=5 before="ui.element" after="level=not-static props=false nested=true native=true" @57:83
rule=pass.hoist-static.fact node=3 before="ui.component" after="level=not-static props=false nested=false native=false" @39:90

"##;

const S3_PAGE: &str = "[s3-folio]
phase=built

[s3-folio.regions]
id=0 parent=- owner=- span=3:90
id=1 parent=0 owner=0 span=21:30
id=2 parent=0 owner=3 span=57:83
id=3 parent=2 owner=5 span=70:72

[s3-folio.ops]
id=0 kind=impeto.insert-node region=0 effect=- span=3:36
id=1 kind=impeto.set-prop region=0 effect=0 span=8:20
id=2 kind=impeto.set-text region=1 effect=1 span=21:30
id=3 kind=impeto.create-component region=0 effect=2 span=39:90
id=4 kind=impeto.set-prop region=0 effect=3 span=45:56
id=5 kind=impeto.insert-node region=2 effect=4 span=57:83
id=6 kind=impeto.slot-outlet region=2 effect=5 span=67:69
id=7 kind=impeto.set-text region=3 effect=6 span=70:72

[s3-folio.edges]
from=1 to=2 kind=effect-order effect=-
from=2 to=3 kind=effect-order effect=-
from=3 to=4 kind=effect-order effect=-
from=4 to=5 kind=effect-order effect=-
from=5 to=6 kind=effect-order effect=-
from=6 to=7 kind=effect-order effect=-
from=0 to=3 kind=dom-order effect=-

[s3-folio.effects]
id=0 owner=1 region=0 span=8:20
id=1 owner=2 region=1 span=21:30
id=2 owner=3 region=0 span=39:90
id=3 owner=4 region=0 span=45:56
id=4 owner=5 region=2 span=57:83
id=5 owner=6 region=2 span=67:69
id=6 owner=7 region=3 span=70:72

";

const S3_PARTITION_PAGE: &str = "[s3-partition-folio]

[s3-partition-folio.ops]
op=0 kind=static span=3:36
op=1 kind=dynamic span=8:20
op=2 kind=dynamic span=21:30
op=3 kind=dynamic span=39:90
op=4 kind=dynamic span=45:56
op=5 kind=dynamic span=57:83
op=6 kind=dynamic span=67:69
op=7 kind=dynamic span=70:72

";

const S3_VALUES_PAGE: &str = r#"[s3-values-folio]

[s3-values-folio.operands]
operand=[0,"tag",null,null,null,"literal","div","",3,36]
operand=[0,"namespace",null,null,null,"literal","html","",3,36]
operand=[1,"name",0,null,null,"literal","class","",8,20]
operand=[1,"value",0,null,null,"js","cls","",16,19]
operand=[1,"binding-kind",0,null,null,"literal","bind","",8,20]
operand=[2,"text",null,null,null,"js","msg","",24,27]
operand=[3,"tag",null,null,null,"literal","Comp","",39,90]
operand=[4,"name",3,null,null,"absent","","",45,56]
operand=[4,"model-read",3,null,null,"js","x","",54,55]
operand=[4,"model-write",3,null,null,"js","x","",54,55]
operand=[4,"model-attribute",3,null,"element-kind","literal","component","",45,56]
operand=[4,"binding-kind",3,null,null,"literal","model","",45,56]
operand=[5,"tag",null,null,null,"literal","template","",57,83]
operand=[5,"namespace",null,null,null,"literal","html","",57,83]
operand=[6,"name",5,null,null,"literal","a","",67,69]
operand=[6,"value",5,null,null,"absent","","",67,69]
operand=[6,"params",5,null,null,"absent","","",67,69]
operand=[6,"binding-kind",5,null,null,"literal","slot-content","",67,69]
operand=[7,"text",null,null,null,"literal","hi","",70,72]

"#;

fn load_schema() -> serde_json::Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("davinci-road")
        .join("plan")
        .join("spolvero-feed.schema.json");
    let text = std::fs::read_to_string(&path).expect("committed schema reads");
    serde_json::from_str(&text).expect("committed schema is valid JSON")
}

fn rungs(pages: &[SpolveroPage]) -> Vec<(&str, &str)> {
    pages
        .iter()
        .map(|page| (page.stage.as_str(), page.pass.as_str()))
        .collect()
}

#[test]
fn the_ladder_validates_and_pins_every_rung_exactly() {
    let pages = ladder_pages("src/App.vue", TEMPLATE);
    let feed = spolvero_value("analyze-sfc", pages.clone());
    assert_eq!(schema_check::validate(&load_schema(), &feed, "$"), Ok(()));

    // The artifact-selected S2 plan: slot carriers and a model binding keep
    // both mandatory barriers, the default profile keeps the optional static
    // and complexity analyses. Every Vue 3 pass preserves the tree, so its page is the
    // lowering's page byte for byte - "which pass changed the folio" is none.
    let expected = [
        ("s1", "parse", TEMPLATE),
        ("s2", "lower", S2_PAGE),
        ("s2-plan", "transform", S2_PLAN_PAGE),
        ("s2", "v-slot", S2_PAGE),
        ("s2", "v-model", S2_PAGE),
        ("s2", "hoist-static", S2_PAGE),
        ("s2", "template-complexity", S2_PAGE),
        ("s2-provenance", "transform", S2_PROVENANCE_PAGE),
        ("s3", "lower", S3_PAGE),
        ("s3-partition", "lower", S3_PARTITION_PAGE),
        ("s3-values", "lower", S3_VALUES_PAGE),
    ];
    let expected_pages: Vec<serde_json::Value> = expected
        .iter()
        .map(|(stage, pass, text)| {
            serde_json::json!({ "path": "src/App.vue", "stage": stage, "pass": pass, "text": text })
        })
        .collect();
    assert_eq!(
        feed,
        serde_json::json!({
            "schema_version": 1,
            "command": "analyze-sfc",
            "pages": expected_pages,
            "remarks": [],
        })
    );
}

#[test]
fn the_s2_pass_pages_follow_the_artifact_selected_plan() {
    // No slot carrier, no model: only the optional analyses run.
    let plain = ladder_pages("src/Plain.vue", "<p>{{ a }}</p>");
    assert_eq!(
        rungs(&plain),
        vec![
            ("s1", "parse"),
            ("s2", "lower"),
            ("s2-plan", "transform"),
            ("s2", "hoist-static"),
            ("s2", "template-complexity"),
            ("s2-provenance", "transform"),
            ("s3", "lower"),
            ("s3-partition", "lower"),
            ("s3-values", "lower"),
        ]
    );
    // A slot carrier alone adds exactly its barrier.
    let slotted = ladder_pages("src/Slot.vue", "<Card><template #head>h</template></Card>");
    assert_eq!(
        rungs(&slotted)[1..5],
        [
            ("s2", "lower"),
            ("s2-plan", "transform"),
            ("s2", "v-slot"),
            ("s2", "hoist-static")
        ]
    );
}

#[test]
fn stage_spans_index_the_s1_page() {
    // The S2/S3 `@start:end` / `span=start:end` offsets are byte offsets into
    // the S1 page text - the authored template - which is what lets a viewer
    // map a selected op back to its source without re-deriving anything.
    let pages = ladder_pages("src/App.vue", TEMPLATE);
    let s1 = pages[0].text.as_str();
    assert_eq!(&s1[24..27], "msg");
    assert_eq!(&s1[21..30], "{{ msg }}");
    assert_eq!(&s1[8..20], ":class=\"cls\"");
    assert_eq!(
        &s1[39..90],
        "<Comp v-model=\"x\"><template #a>hi</template></Comp>"
    );
    assert_eq!(&s1[70..72], "hi");
}

#[test]
fn a_malformed_template_still_climbs_every_rung() {
    // Every stage is total: typed holes in S1, kept fragments in S2, and an
    // S3 graph over whatever S2 kept. The S1 page is still the authored bytes.
    let template = "\n<div class=\"open>{{ msg }\n";
    let pages = ladder_pages("src/Broken.vue", template);
    let s2 = "[disegno]\nops=1\n\n[disegno.ops]\nui.element div @1:27\n  attr class=\"open>{{ msg }\\n\" @6:27\n\n";
    assert_eq!(
        pages
            .iter()
            .map(|page| (page.stage.as_str(), page.pass.as_str(), page.text.as_str()))
            .collect::<Vec<_>>(),
        vec![
            ("s1", "parse", template),
            ("s2", "lower", s2),
            (
                "s2-plan",
                "transform",
                "[fusion-plan-folio]\nstage=s2\nwalks=1\n\n[fusion-plan-folio.passes]\nwalk=0 pass=hoist-static kind=optional fusability=fusable\nwalk=0 pass=template-complexity kind=optional fusability=fusable\n\n",
            ),
            ("s2", "hoist-static", s2),
            ("s2", "template-complexity", s2),
            (
                "s2-provenance",
                "transform",
                "[s2-provenance-folio]\n\n[s2-provenance-folio.records]\nrule=condense.drop-whitespace node=- before=\"\\n\" after=\"\" @0:1\nrule=lower.element node=0 before=\"<div class=\\\"open>{{ msg }\\n\" after=\"ui.element div\" @1:27\nrule=pass.hoist-static.fact node=0 before=\"ui.element\" after=\"level=fully-static props=true nested=false native=true\" @1:27\n\n",
            ),
            (
                "s3",
                "lower",
                "[s3-folio]\nphase=built\n\n[s3-folio.regions]\nid=0 parent=- owner=- span=1:27\nid=1 parent=0 owner=0 span=1:27\n\n[s3-folio.ops]\nid=0 kind=impeto.insert-node region=0 effect=- span=1:27\n\n",
            ),
            (
                "s3-partition",
                "lower",
                "[s3-partition-folio]\n\n[s3-partition-folio.ops]\nop=0 kind=static span=1:27\n\n",
            ),
            (
                "s3-values",
                "lower",
                "[s3-values-folio]\n\n[s3-values-folio.operands]\noperand=[0,\"tag\",null,null,null,\"literal\",\"div\",\"\",1,27]\noperand=[0,\"namespace\",null,null,null,\"literal\",\"html\",\"\",1,27]\noperand=[0,\"attribute\",null,null,\"class\",\"literal\",\"open>{{ msg }\\n\",\"\",6,27]\n\n",
            ),
        ]
    );
}
