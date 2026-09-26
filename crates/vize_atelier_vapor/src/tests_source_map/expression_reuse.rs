//! Equal expression bytes at different positions retain separate map anchors.

use super::support::{Lane, mapped_on};

#[test]
fn repeated_expressions_keep_authored_maps_in_each_scope() {
    for source in [
        r#"<main><p :id="item.value"></p><p :title="item.value"></p></main>"#,
        r#"<main><p :id="item.value"></p><p v-for="item in items" :title="item.value"></p><p :title="item.value"></p></main>"#,
        r#"<main><button @click="save" :title="label"></button><button @click="save" :title="label"></button></main>"#,
    ] {
        assert_eq!(
            mapped_on(source, Lane::Selected),
            mapped_on(source, Lane::Legacy),
            "{source}"
        );
    }
}
