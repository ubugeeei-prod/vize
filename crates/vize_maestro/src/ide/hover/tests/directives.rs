//! Hover over authored directive attribute names in a realistic template.

use tower_lsp::lsp_types::HoverContents;

use super::super::HoverService;
use super::state_with_document;
use crate::ide::IdeContext;

const SOURCE: &str = r#"<script setup lang="ts">
import { ref } from 'vue'
const name = ref('')
const items = ref([{ id: 1, label: 'One' }])
const attrs = { role: 'list' }
function save(event: Event) {}
</script>

<template>
  <form @submit.prevent="save" v-on:reset="name = ''">
    <input v-model.trim="name" :title="name" v-bind:aria-label="name" />
    <ul v-bind="attrs" v-if="items.length">
      <li v-for="item in items" :key="item.id">{{ item.label }}</li>
    </ul>
    <MyList>
      <template #header>Head</template>
      <template v-slot:item="{ item }">{{ item }}</template>
    </MyList>
    <p v-show="name" @update:model-value="save">x</p>
  </form>
</template>
"#;

/// The first two markdown sections (title and meta line) of the hover at the
/// `nth` byte inside the first occurrence of `needle`.
fn hover_head(needle: &str, nth: usize) -> Option<[String; 2]> {
    let (state, uri) = state_with_document("Directives.vue", SOURCE);
    let offset = SOURCE.find(needle).unwrap() + nth;
    let ctx = IdeContext::new(&state, &uri, offset).unwrap();
    let hover = HoverService::hover(&ctx)?;
    let HoverContents::Markup(markup) = hover.contents else {
        panic!("expected markdown hover");
    };
    let mut sections = markup.value.split("\n\n").map(str::to_owned);
    Some([sections.next()?, sections.next().unwrap_or_default()])
}

#[test]
fn directive_attribute_names_hover_with_their_directive_card() {
    for (needle, nth, title, meta) in [
        ("@submit.prevent", 3, "**@submit**", "_Vue event listener_"), // event name
        ("@submit.prevent", 10, "**@submit**", "_Vue event listener_"), // modifier
        ("v-on:reset", 7, "**@reset**", "_Vue event listener_"),       // long-form argument
        (
            "@update:model-value",
            12,
            "**@update:model-value**",
            "_Vue event listener_",
        ), // namespaced event
        ("v-model.trim", 9, "**v-model**", "_Vue template directive_"), // modifier
        (
            "v-bind=\"attrs\"",
            3,
            "**v-bind**",
            "_Vue template directive_",
        ), // object spread
        ("v-if", 2, "**v-if**", "_Vue template directive_"),
        ("v-for", 3, "**v-for**", "_Vue template directive_"),
        ("v-show", 3, "**v-show**", "_Vue template directive_"),
        ("#header", 3, "**v-slot**", "_Vue template directive_"), // shorthand
        ("v-slot:item", 9, "**v-slot**", "_Vue template directive_"), // argument
        (":title", 2, "**title on <input>**", "_HTML attribute_"), // native bound attribute
        (
            "v-bind:aria-label",
            10,
            "**aria-label on <input>**",
            "_HTML attribute_",
        ), // native long-form argument
    ] {
        assert_eq!(
            hover_head(needle, nth),
            Some([title.to_owned(), meta.to_owned()]),
            "{needle} @{nth}"
        );
    }
}

#[test]
fn expressions_inside_directives_hover_the_binding_not_the_directive() {
    for (needle, nth, title, meta) in [
        ("=\"save\" v-on", 3, "**save**", "_Template binding_"), // handler expression
        ("name = ''", 1, "**name**", "_Template binding_"),      // inline handler
        ("items.length", 2, "**items**", "_Template binding_"),  // v-if expression
        ("{{ item.label", 4, "**item**", "_v-for scope binding_"), // v-for alias
    ] {
        assert_eq!(
            hover_head(needle, nth),
            Some([title.to_owned(), meta.to_owned()]),
            "{needle} @{nth}"
        );
    }
}

const EDGE: &str = r#"<script setup lang="ts">
const key = 'title'
const ev = 'click'
const value = 1
</script>

<template>
  <div .textContent="value" v-bind:[key]="value" v-on:[ev]="value" @click.once="value" v-html="value" v-memo="[value]" v-once v-cloak />
</template>
"#;

#[test]
fn edge_directive_forms_hover_with_their_directive_card() {
    let (state, uri) = state_with_document("Edge.vue", EDGE);
    for (needle, nth, expected) in [
        // `.prop` shorthand is `v-bind` with the `.prop` modifier.
        (
            ".textContent",
            3,
            Some(["**v-bind**", "_Vue attribute / prop binding_"]),
        ),
        (
            "v-bind:[key]",
            3,
            Some(["**v-bind**", "_Vue attribute / prop binding_"]),
        ),
        ("v-bind:[key]", 9, Some(["**key**", "_Template binding_"])),
        ("v-on:[ev]", 2, Some(["**@[ev]**", "_Vue event listener_"])),
        ("v-on:[ev]", 7, Some(["**ev**", "_Template binding_"])),
        (
            "@click.once",
            8,
            Some(["**@click**", "_Vue event listener_"]),
        ),
        (
            "v-html",
            3,
            Some(["**v-html**", "_Vue template directive_"]),
        ),
        (
            "v-memo",
            3,
            Some(["**v-memo**", "_Vue template directive_"]),
        ),
        (
            "v-once",
            3,
            Some(["**v-once**", "_Vue template directive_"]),
        ),
        (
            "v-cloak",
            3,
            Some(["**v-cloak**", "_Vue template directive_"]),
        ),
    ] {
        let offset = EDGE.find(needle).unwrap() + nth;
        let ctx = IdeContext::new(&state, &uri, offset).unwrap();
        let head = HoverService::hover(&ctx).map(|hover| {
            let HoverContents::Markup(markup) = hover.contents else {
                panic!("expected markdown hover");
            };
            let mut sections = markup.value.split("\n\n").map(str::to_owned);
            [
                sections.next().unwrap_or_default(),
                sections.next().unwrap_or_default(),
            ]
        });
        assert_eq!(
            head,
            expected.map(|pair| pair.map(str::to_owned)),
            "{needle} @{nth}"
        );
    }
}
