//! Conditional and loop carrier names/expressions keep exact authored mappings.

use super::support::{Lane, mapped_on};

#[test]
fn structural_slot_code_and_maps_match_both_lanes() {
    for (case, source) in [
        (
            "conditional",
            r#"<Child><template #[names[selected]]="p" v-if="enabled"><b>{{p.x}}</b></template><template #two v-else>B</template></Child>"#,
        ),
        (
            "loop",
            r#"<Child><template v-for="(item, key) in items" #[item.name]="p"><button @click="record(item.label)">{{item.label}}:{{key}}:{{p.x}}</button></template></Child>"#,
        ),
    ] {
        let (code, segments) = mapped_on(source, Lane::Selected);
        assert_eq!(
            mapped_on(source, Lane::Legacy),
            (code.clone(), segments.clone()),
            "{case}"
        );
        insta::assert_snapshot!(format!("structural_slot_{case}_code"), code);
        insta::assert_debug_snapshot!(format!("structural_slot_{case}_map"), segments);
    }
}
