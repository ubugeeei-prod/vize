//! Component, built-in, and slot-outlet fixtures for the SSR emitter
//! differential.

/// Templates the string plan must own.
pub(super) const ADMITTED: &[(&str, &str)] = &[
    ("component-bare", "<Foo />"),
    (
        "component-root-props",
        r#"<Foo id="a" :title="title" class="c" :class="k" />"#,
    ),
    (
        "component-nested",
        r#"<div><Foo a="1" /><Bar :b="b" /></div>"#,
    ),
    (
        "component-kebab",
        r#"<div><my-widget data-x="1" :value="v" /></div>"#,
    ),
    (
        "component-events",
        r#"<Foo @click="onClick" @custom-event="onClick" @bump="count++" @key-up.enter="onKey" />"#,
    ),
    (
        "component-handlers",
        r#"<div><Foo @a="() => go(1)" @b="function () { go(2) }" @c="obj.fn" @d="go($event)" @e="a++; b++" /></div>"#,
    ),
    (
        "component-handler-entity",
        r#"<div><Foo @a="a &amp;&amp; b()" /></div>"#,
    ),
    ("component-on-object", r#"<Foo v-on="handlers" :id="id" />"#),
    (
        "component-spread",
        r#"<Foo v-bind="attrs" :[name]="value" />"#,
    ),
    (
        "component-spread-mid",
        r#"<div><Foo a="1" v-bind="rest" b="2" /></div>"#,
    ),
    (
        "component-bind-modifiers",
        r#"<div><Foo :title.camel="t" .prop="p" :x.attr="x" /></div>"#,
    ),
    (
        "component-dynamic-on",
        r#"<div><Foo @[evt]="handler" /></div>"#,
    ),
    ("component-model", r#"<Foo v-model="value" />"#),
    (
        "component-show",
        r#"<MyComp style="color: red;" v-show="ok" />"#,
    ),
    (
        "component-ignored-directives",
        r#"<div><Foo v-focus="x" v-once v-text="t" /></div>"#,
    ),
    (
        "component-in-if",
        r#"<Foo v-if="a" :x="1" /><Bar v-else />"#,
    ),
    (
        "component-in-for",
        r#"<ul><Item v-for="item in items" :key="item.id" :item="item" @pick="select(item)" /></ul>"#,
    ),
    ("component-self-name", r#"<div><Fixture /><Other /></div>"#),
    (
        "teleport",
        r#"<Teleport to="body"><div v-if="ok">tip</div></Teleport>"#,
    ),
    (
        "teleport-bound",
        r#"<div><Teleport :to="target" :disabled="off"><p>{{ msg }}</p></Teleport></div>"#,
    ),
    (
        "teleport-bare",
        r#"<div><teleport disabled><span>x</span></teleport></div>"#,
    ),
    ("suspense", r#"<Suspense><div>{{ a }}</div></Suspense>"#),
    (
        "suspense-component",
        r#"<div><Suspense><Foo /></Suspense></div>"#,
    ),
    (
        "transition-root",
        r#"<Transition><Foo label="ready" /></Transition>"#,
    ),
    (
        "transition-element",
        r#"<Transition name="fade"><div v-if="show" class="x">hi</div></Transition>"#,
    ),
    (
        "keep-alive",
        r#"<div><KeepAlive><Foo :k="k" /></KeepAlive></div>"#,
    ),
    ("slot-basic", r#"<div><slot /></div>"#),
    ("slot-root", r#"<slot name="header" :item="item" />"#),
    (
        "slot-fallback",
        r#"<div><slot name="a" :x="y">fb {{ z }}</slot></div>"#,
    ),
    (
        "slot-props",
        r#"<div><slot data-id="1" :item-key="k" v-bind="extra" bare /></div>"#,
    ),
    (
        "slot-dynamic-name",
        r#"<div><slot :name="slotName" /></div>"#,
    ),
    (
        "slot-in-for",
        r#"<ul><li v-for="item in items"><slot :item="item">{{ item.label }}</slot></li></ul>"#,
    ),
    ("slot-with-on", r#"<div><slot @click="go" :a="a" /></div>"#),
    (
        "slot-in-v-for",
        r#"<div><slot v-for="n in 3" :n="n" /></div>"#,
    ),
    (
        "component-v-if-branch",
        r#"<Foo v-if="a" /><p v-else>b</p>"#,
    ),
];
