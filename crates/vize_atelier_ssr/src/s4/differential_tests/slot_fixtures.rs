//! Component slot content fixtures: the push form and the VNode fallback.

/// Templates the string plan must own.
pub(super) const ADMITTED: &[(&str, &str)] = &[
    ("slot-text", r#"<Foo>hello</Foo>"#),
    ("slot-element", r#"<Foo><div>slot content</div></Foo>"#),
    (
        "slot-mixed",
        r#"<Foo a="1">x {{ y }} <b :title="t">z</b></Foo>"#,
    ),
    (
        "slot-named",
        r#"<ClientOnly><span>client</span><template #fallback><span>server</span></template></ClientOnly>"#,
    ),
    (
        "slot-named-only",
        r#"<Foo><template #header><h1>{{ title }}</h1></template><template #footer>bye</template></Foo>"#,
    ),
    (
        "slot-scoped",
        r#"<Foo><template #item="{ item, index }"><li :data-i="index">{{ item.name }}</li></template></Foo>"#,
    ),
    (
        "slot-scoped-default",
        r#"<List v-slot="{ row }"><span>{{ row.id }} {{ other }}</span></List>"#,
    ),
    (
        "slot-kebab-name",
        r#"<Foo><template #row-item="props"><i>{{ props.x }}</i></template></Foo>"#,
    ),
    (
        "slot-nested",
        r#"<Outer><Inner :x="1"><b>{{ deep }}</b></Inner></Outer>"#,
    ),
    (
        "slot-nested-scoped",
        r#"<Outer v-slot="{ a }"><Inner><template #default="{ b }">{{ a }}{{ b }}</template></Inner></Outer>"#,
    ),
    (
        "slot-if",
        r#"<Foo><p v-if="ok">yes</p><p v-else-if="maybe">?</p><p v-else>no</p></Foo>"#,
    ),
    (
        "slot-if-template",
        r#"<Foo><template v-if="ok"><a>1</a><b>2</b></template><i v-else>x</i></Foo>"#,
    ),
    (
        "slot-if-no-else",
        r#"<Foo><span v-if="show">s</span></Foo>"#,
    ),
    (
        "slot-for",
        r#"<Foo><li v-for="item in items" :key="item.id">{{ item.label }}</li></Foo>"#,
    ),
    (
        "slot-for-unkeyed",
        r#"<Foo><li v-for="(item, i) in items">{{ i }}</li></Foo>"#,
    ),
    (
        "slot-for-template-keyed",
        r#"<Foo><template v-for="x in xs" :key="x.id"><dt>{{ x.a }}</dt><dd>{{ x.b }}</dd></template></Foo>"#,
    ),
    (
        "slot-for-template-single",
        r#"<Foo><template v-for="x in xs" :key="x"><em>{{ x }}</em></template></Foo>"#,
    ),
    (
        "slot-for-template-unkeyed",
        r#"<Foo><template v-for="x in xs"><em>{{ x }}</em><i>!</i></template></Foo>"#,
    ),
    (
        "slot-outlet-forward",
        r#"<Wrapper><slot name="inner" :v="v" /></Wrapper>"#,
    ),
    (
        "slot-outlet-fallback",
        r#"<Wrapper><slot>fallback {{ z }}</slot></Wrapper>"#,
    ),
    (
        "slot-outlet-in-scope",
        r#"<Wrapper v-slot="{ s }"><Inner><slot :s="s" /></Inner></Wrapper>"#,
    ),
    (
        "slot-component-props",
        r#"<Card><Btn @click="go" :disabled="busy" v-bind="extra">Go</Btn></Card>"#,
    ),
    (
        "slot-attrs-vnode",
        r#"<Foo><input v-model="m" :value="v" class="c" id="i"><p v-show="s" v-html="h"></p></Foo>"#,
    ),
    (
        "slot-interpolation-run",
        r#"<Foo>a {{ b }} c {{ d }}</Foo>"#,
    ),
    (
        "slot-entities",
        r#"<Foo>&lt;tag&gt; &amp; "q"<span title="a&amp;b">x</span></Foo>"#,
    ),
    (
        "slot-builtin-in-slot",
        r#"<Foo><Transition><div v-if="a">t</div></Transition></Foo>"#,
    ),
    (
        "slot-teleport-in-slot",
        r#"<Foo><Teleport to="body"><p>x</p></Teleport></Foo>"#,
    ),
    (
        "slot-in-for",
        r#"<ul><Row v-for="r in rows" :key="r.id"><td>{{ r.v }}</td></Row></ul>"#,
    ),
    (
        "slot-root-fallthrough",
        r#"<Layout class="page"><main>{{ body }}</main></Layout>"#,
    ),
    (
        "slot-padded-pattern",
        r#"<Foo><template #a=" { x } ">{{ x }}</template></Foo>"#,
    ),
    (
        "slot-svg",
        r#"<Icon><svg viewBox="0 0 1 1"><path :d="d"/></svg></Icon>"#,
    ),
    (
        "slot-if-keyed-component",
        r#"<Foo><Bar v-if="a" :key="k" :x="1" /><Baz v-else key="z" /></Foo>"#,
    ),
    (
        "slot-if-keyed-element",
        r#"<Foo><p v-if="a" :key="k" :title="t">1</p><p v-else :key="j">2</p></Foo>"#,
    ),
    (
        "slot-if-keyed-outlet",
        r#"<Foo><slot v-if="a" :key="k" :v="v" /></Foo>"#,
    ),
    (
        "create-slots-mixed",
        r#"<Foo><template v-if="a" #a="{ p }">x{{ p }}</template><template v-else-if="b" #b>y</template><template v-for="n in names" #[n]="s">{{ s }}{{ n }}</template><template #[dyn]>d</template><template #st>s</template>z</Foo>"#,
    ),
    (
        "create-slots-conditional-else",
        r#"<Foo><template v-if="a" #a>1</template><template v-else #b>2</template></Foo>"#,
    ),
    (
        "create-slots-conditional-plain-branch",
        r#"<Foo><template v-if="a" #a>1</template><p v-else>plain</p></Foo>"#,
    ),
    (
        "create-slots-looped-aliases",
        r#"<Foo><template v-for="({ id, label }, i) in cols" #[id]="{ row }">{{ row[id] }} {{ label }} {{ i }}</template></Foo>"#,
    ),
    (
        "create-slots-looped-static-name",
        r#"<Foo><template v-for="n in 3" #cell>{{ n }}</template></Foo>"#,
    ),
    (
        "create-slots-in-slot",
        r#"<Bar><Foo><template v-if="a" #a>x</template><template #[k]="{ v }"><i>{{ v }}</i></template></Foo></Bar>"#,
    ),
    (
        "create-slots-in-scope",
        r#"<Bar v-slot="{ s }"><Foo><template v-for="n in s.names" #[n]><slot :name="n" /></template></Foo></Bar>"#,
    ),
    (
        "create-slots-dynamic-component",
        r#"<component :is="c"><template v-if="a" #head>h</template>body</component>"#,
    ),
    (
        "slot-template-plain",
        r#"<Foo><template><b>{{ x }}</b></template><iframe :src="u"></iframe></Foo>"#,
    ),
    (
        "root-if-keyed",
        r#"<div v-if="a" :key="k" :id="i">x</div><Bar v-else :key="k2" :y="2" />"#,
    ),
];
