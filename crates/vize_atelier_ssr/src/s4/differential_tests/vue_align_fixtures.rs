//! Shapes the `fix(ssr)!` Vue 3.5 alignment moved onto Vue's merged-props
//! path: custom directives, `v-once` / `v-cloak` / `v-memo`, spreads and
//! dynamic keys, `.camel`, the moved `v-show`, and scoped dynamic keys.

/// Templates the string plan must own, in parity with the aligned legacy lane.
pub(super) const ADMITTED: &[(&str, &str)] = &[
    (
        "directive-inline",
        r#"<div><p v-focus class="a">t</p></div>"#,
    ),
    (
        "directive-value-arg-modifiers",
        r#"<div><p v-focus:a.b.c-d="x" :id="i">t</p></div>"#,
    ),
    (
        "directive-dynamic-arg",
        r#"<div><p v-focus:[side]="x">t</p></div>"#,
    ),
    (
        // Under binding metadata the transform prefixes a directive argument
        // (`$props.title`, `$setup.count`) but leaves a bind key unprefixed.
        "directive-bound-dynamic-args",
        r#"<div><p v-focus:[title]="x">t</p><p v-a:[count]></p><p v-b:[label] :[title]="1">t</p><p v-c:[Math]>t</p></div>"#,
    ),
    (
        "directive-dynamic-arg-in-for",
        r#"<ul><li v-for="k in ks" v-focus:[k]>{{ k }}</li></ul>"#,
    ),
    (
        "directive-modifiers-only",
        r#"<div><p v-pin.top>t</p></div>"#,
    ),
    (
        "directive-empty-content",
        r#"<div><p v-content></p><img v-lazy="src"><input v-focus></div>"#,
    ),
    (
        "directive-content-override",
        r#"<div><p v-focus v-html="h"></p></div>"#,
    ),
    (
        "directive-root",
        r#"<section class="r" v-focus="x" v-show="s">x</section>"#,
    ),
    (
        "directive-twice",
        r#"<div><p v-a="1" v-b:c="2">t</p></div>"#,
    ),
    (
        "directive-in-for",
        r#"<ul><li v-for="i in items" v-focus="i">{{ i }}</li></ul>"#,
    ),
    (
        "builtins-dropped",
        r#"<div><p v-once>{{ a }}</p><p v-cloak>{{ b }}</p><p v-memo="[c]">{{ c }}</p></div>"#,
    ),
    ("builtin-root", r#"<div v-once>{{ a }}</div>"#),
    (
        "camel-inline",
        r#"<div><svg :view-box.camel="vb"></svg></div>"#,
    ),
    (
        "prop-attr-inline",
        r#"<div><p :title.prop="t" :data-x.attr="d"></p></div>"#,
    ),
    (
        "camel-root",
        r#"<svg :view-box.camel="vb" :data-a.attr="a"></svg>"#,
    ),
    (
        "dynamic-key-inline",
        r#"<div><p :[k]="v" class="a">t</p></div>"#,
    ),
    (
        "dynamic-key-in-for",
        r#"<div><p v-for="k in ks" :[k]="1">t</p></div>"#,
    ),
    (
        "spread-inline",
        r#"<div><p class="a" id="first" v-bind="obj" :title="t">t</p></div>"#,
    ),
    ("spread-only-inline", r#"<div><p v-bind="obj">t</p></div>"#),
    (
        "spread-root-order",
        r#"<div id="a" v-bind="obj" :title="t">x</div>"#,
    ),
    (
        "spread-two",
        r#"<div><p v-bind="a" id="x" v-bind="b">t</p></div>"#,
    ),
    (
        "spread-input-model",
        r#"<div><input v-bind="o" v-model="m"></div>"#,
    ),
    (
        "spread-input-typed-model",
        r#"<div><input type="checkbox" v-bind="o" v-model="m"></div>"#,
    ),
    (
        "spread-textarea",
        r#"<div><textarea v-bind="o">fallback</textarea></div>"#,
    ),
    (
        "spread-textarea-model",
        r#"<div><textarea v-bind="o" v-model="m"></textarea></div>"#,
    ),
    (
        "show-root-after-attrs",
        r#"<div v-show="s" class="a" style="color:red">x</div>"#,
    ),
    (
        "show-spread",
        r#"<div><p :class="c" v-bind="o" style="color:red" v-show="s">t</p></div>"#,
    ),
    ("outlet-dynamic-key", r#"<div><slot :[k]="v" /></div>"#),
    (
        "outlet-dynamic-key-in-for",
        r#"<div><slot v-for="k in ks" :[k]="1" /></div>"#,
    ),
    (
        "component-dynamic-key-in-for",
        r#"<div><Foo v-for="k in ks" :[k]="1" /></div>"#,
    ),
    (
        "component-dynamic-key-in-slot-scope",
        r#"<List v-slot="{ key }"><Foo :[key]="1" /></List>"#,
    ),
    (
        "directive-in-slot",
        r#"<Foo><p v-focus="x">in slot</p><i v-content></i></Foo>"#,
    ),
];
