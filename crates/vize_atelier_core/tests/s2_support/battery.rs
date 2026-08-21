//! The committed battery the P2-9 comparator runs — the same sources in
//! the plain witness and the corpus entry, so a feature-wiring
//! regression fails loudly in both lanes. Series 1 contributed the
//! v-if templates; series 2 the v-for half; series 3 the v-slot half;
//! series 4 the text half.

/// (name, template source). Each names the projection class it pins.
#[cfg_attr(feature = "legacy", allow(dead_code))]
pub const BATTERY: &[(&str, &str)] = &[
    ("simple-if", r#"<div v-if="show">on</div>"#),
    (
        "if-else",
        r#"<div v-if="show">on</div><div v-else>off</div>"#,
    ),
    (
        "full-chain",
        r#"<a v-if="x === 1">1</a><b v-else-if="x === 2">2</b><i v-else>other</i>"#,
    ),
    (
        "static-keys",
        r#"<p v-if="a" key="one">1</p><p v-else-if="b" key="two">2</p><p v-else key="three">3</p>"#,
    ),
    (
        "duplicate-keys",
        r#"<p v-if="a" key="dup">1</p><p v-else key="dup">2</p>"#,
    ),
    ("bare-keys", r#"<p v-if="a" key>1</p><p v-else key>2</p>"#),
    (
        "dynamic-keys",
        r#"<p v-if="a" :key="ka">1</p><p v-else :key="kb">2</p>"#,
    ),
    (
        "template-if",
        r#"<template v-if="a"><p>1</p><p>2</p></template><div v-else>3</div>"#,
    ),
    (
        "template-if-key",
        r#"<template v-if="a" key="t"><p>1</p></template><div v-else key="d">2</div>"#,
    ),
    (
        "template-child-key",
        r#"<template v-if="a"><div key="z">1</div></template><p v-else>2</p>"#,
    ),
    (
        "if-and-for",
        r#"<li v-if="ready" v-for="item in items" key="row">{{ item }}</li>"#,
    ),
    (
        "nested-chains",
        r#"<div v-if="outer"><span v-if="inner" key="i">a</span><span v-else>b</span></div><div v-else>c</div>"#,
    ),
    (
        "comment-gap",
        "<p v-if=\"a\">1</p><!-- between branches -->\n<p v-else>2</p>",
    ),
    (
        "component-branches",
        r#"<Panel v-if="a" key="p">x</Panel><Drawer v-else key="d">y</Drawer>"#,
    ),
    (
        "slot-outlet-key",
        r#"<slot v-if="a" key="s"></slot><p v-else>fallback</p>"#,
    ),
    (
        "padded-condition",
        "<div v-if=\"  ready && !done  \">padded</div>",
    ),
    (
        "multiline-condition",
        "<div v-if=\"mode === 'a' ||\n  mode === 'b'\">m</div>",
    ),
    (
        "deep-siting",
        r#"<ul><li v-for="i in xs"><em v-if="i.ok" key="ok">y</em><em v-else key="no">n</em></li></ul>"#,
    ),
    // -- the v-for half (series 2) ----------------------------------
    ("simple-for", r#"<li v-for="item in items">{{ item }}</li>"#),
    (
        "for-of-pair",
        r#"<li v-for="(item, i) of items">{{ i }}</li>"#,
    ),
    (
        "for-three-aliases",
        r#"<td v-for="(value, key, index) in obj">{{ key }}</td>"#,
    ),
    (
        "for-destructure",
        r#"<tr v-for="({ id, name }, i) in rows">{{ id }}</tr>"#,
    ),
    (
        "for-no-parens",
        r#"<li v-for="item, index in items">{{ index }}</li>"#,
    ),
    (
        "template-for",
        r#"<template v-for="n in count"><i>{{ n }}</i><b>fixed</b></template>"#,
    ),
    (
        "for-keyed-element",
        r#"<li v-for="item in items" key="row">{{ item }}</li>"#,
    ),
    ("for-absent-alias", r#"<div v-for=" in xs">filler</div>"#),
    (
        "for-first-viable-in",
        r#"<p v-for="a in b in c">{{ a }}</p>"#,
    ),
    (
        "nested-for",
        r#"<ul v-for="row in grid"><li v-for="cell in row">{{ cell }}</li></ul>"#,
    ),
    ("for-on-slot", r#"<slot v-for="x in xs"></slot>"#),
    (
        "if-for-precedence",
        r#"<li v-if="ready" v-for="i in xs">{{ i }}</li><li v-else>empty</li>"#,
    ),
    // -- the v-slot half (series 3) ----------------------------------
    (
        "named-slots",
        r#"<Card><template #head="{ icon }"><b>t</b></template><template v-slot:body="row"><p>{{ row }}</p></template></Card>"#,
    ),
    (
        "self-slot-default",
        r#"<Panel v-slot="props"><em>{{ props.x }}</em></Panel>"#,
    ),
    (
        "authored-default-name",
        r#"<Panel v-slot:default="p">{{ p }}</Panel>"#,
    ),
    (
        "dynamic-slot-name",
        r#"<List><template #[cell]="value">{{ value }}</template></List>"#,
    ),
    (
        "slot-name-modifiers",
        r#"<Box><template #head.raw>x</template></Box>"#,
    ),
    (
        "implicit-default-beside-named",
        r#"<Card><template #a>x</template><p>body</p></Card>"#,
    ),
    (
        "duplicate-slot-names",
        r#"<Grid><template #cell="c">a</template><template #cell>b</template></Grid>"#,
    ),
    (
        "mixed-usage",
        r#"<Modal v-slot="own"><template #late>y</template></Modal>"#,
    ),
    (
        "extraneous-children",
        r#"<Grid><template #default>c</template><hr></Grid>"#,
    ),
    (
        "outlet-names",
        r#"<slot></slot><slot name="s"><b>f</b></slot><slot :name="n"></slot>"#,
    ),
    ("bare-name-outlet", r#"<slot name></slot>"#),
    (
        "comment-filler-default",
        r#"<List><template #row>r</template><!-- note --></List>"#,
    ),
    (
        "conditional-carrier",
        r#"<Tabs><template #fixed>f</template><template v-if="ok" #maybe>m</template></Tabs>"#,
    ),
    (
        "whitespace-only-default",
        "<Card>\n  <template #a>x</template>\n</Card>",
    ),
    // -- the text half (series 4) -----------------------------------
    ("plain-merge", r#"<p>Hello {{ name }}!</p>"#),
    ("interpolation-only", r#"<p>{{ msg }}</p>"#),
    ("static-only", r#"<p>just text</p>"#),
    ("interior-condense", "<p>a   b {{ x }}\n   c</p>"),
    ("comment-run-boundary", r#"<p>a<!--c-->b {{ x }}</p>"#),
    ("comment-remove", "<p>a<!--c-->\n<!--d-->b</p>"),
    ("kept-space", r#"<i>one</i> <i>two</i>"#),
    ("dropped-newline", "<i>one</i>\n<i>two</i>"),
    ("pre-content", r#"<pre>  a  {{ x }}  b  </pre>"#),
    (
        "rawtext-excluded",
        r#"<textarea> a   b </textarea><p>t {{ y }}</p>"#,
    ),
    ("entity-class", r#"<p>Tom &amp; Jerry {{ x }}</p>"#),
    (
        "vpre-class",
        r#"<div v-pre>{{ raw }}</div><p>after {{ x }}</p>"#,
    ),
    (
        "units-in-branches",
        r#"<div v-if="a">x {{ b }}</div><div v-else>y</div>"#,
    ),
    ("units-in-loop", r#"<li v-for="i in xs">#{{ i }} </li>"#),
    // -- the binding-surface half (series 5) --------------------------
    (
        "bind-forms",
        r#"<a :href="url" :[attr]="val" v-bind="rest">x</a>"#,
    ),
    (
        "bind-shorthands",
        r#"<i :model-value .innerHTML="h" :data-x="1">t</i>"#,
    ),
    (
        "on-forms",
        r#"<button @click.stop.prevent="go()" @[evt]="run" v-on="handlers" @press>x</button>"#,
    ),
    ("model-native", r#"<input v-model.lazy.trim="draft.note">"#),
    ("model-component", r#"<Field v-model="doc.title"></Field>"#),
    (
        "model-component-arg-mods",
        r#"<Field v-model:title.trim="doc.title"></Field>"#,
    ),
    (
        "model-invalid-scope",
        r#"<li v-for="item in xs"><input v-model="item"></li>"#,
    ),
    ("model-invalid-arg", r#"<input v-model:value="name">"#),
    ("model-dynamic-arg", r#"<Comp v-model:[k]="x"></Comp>"#),
    (
        "model-pattern-scope",
        r#"<Card v-slot="{ row }"><input v-model="row"></Card>"#,
    ),
    (
        "classifier-ambiguous-model",
        r#"<foobar v-model="x">y</foobar>"#,
    ),
    ("custom-directive", r#"<p v-pin:top.lazy="offset">x</p>"#),
    (
        "outlet-props",
        r#"<slot name="cell" tone="brisk" :item="row" @pick="choose(row)"></slot>"#,
    ),
    ("deferred-builtins", r#"<p v-show="open" v-once>x</p>"#),
    // A duplicate attribute is a hard legacy parse error
    // (`DuplicateAttribute` — the vue2-elm corpus skip), so the #958
    // dedupe question stays outside both lanes' shared domain; plain
    // statics pin the attr surface instead.
    ("static-attrs", r#"<a id="top" data-x="1" title="hi">x</a>"#),
    ("entity-attr", r#"<a title="Tom &amp; Jerry" :x="1">x</a>"#),
    (
        "wrapper-dynamic-key",
        r#"<template v-if="a" :key="wk"><p>1</p></template><p v-else>2</p>"#,
    ),
    // -- the hoist-decision half (series 6) ---------------------------
    (
        "hoist-root-props",
        r#"<div id="page" class="wrap"><b>t</b></div>"#,
    ),
    (
        "hoist-root-dynamic-text-props",
        r#"<p tabindex="0">greeting {{ user }}</p>"#,
    ),
    (
        "hoist-quiet-nesting",
        r#"<div><i>{{ a }}</i><section data-x="1"><b>s</b></section></div>"#,
    ),
    (
        "hoist-under-directive-parent",
        r#"<div v-pin="cfg"><section><b>s</b></section></div>"#,
    ),
    (
        "hoist-under-component-parent",
        r#"<Card><section><b>s</b></section></Card>"#,
    ),
    (
        "hoist-in-branches",
        r#"<div v-if="a"><p><b>x</b></p></div><div v-else>d</div>"#,
    ),
    (
        "hoist-for-carrier",
        r#"<li v-for="i in xs"><em>fixed</em></li>"#,
    ),
    (
        "hoist-svg-foreign-props",
        r#"<svg :width="100"><rect fill="red"></rect></svg>"#,
    ),
    ("hoist-ref-blocker", r#"<div ref="el">x</div>"#),
    (
        "hoist-const-bind",
        r#"<i :tabindex="4 + 4" data-a="b">t</i>"#,
    ),
    (
        "hoist-comment-blocker",
        r#"<div><span>a</span><!-- keep --></div>"#,
    ),
    (
        "hoist-builtin-subtree",
        r#"<article v-memo="[a]"><p><b>y</b></p></article>"#,
    ),
    ("hoist-consts-conservative", r#"<em :max="Math.PI">m</em>"#),
    ("hoist-classifier-skip", r#"<foobar><b>x</b></foobar>"#),
    (
        "hoist-wrapper-props",
        r#"<template v-for="i in 3" data-x="1"><b>c</b></template>"#,
    ),
];
