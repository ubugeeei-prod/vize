//! The plain-element fixture battery for the SSR emitter differential.

/// Templates the string plan must own.
pub(super) const ADMITTED: &[(&str, &str)] = &[
    ("title-text", "<title>Hello world</title>"),
    (
        "title-interpolation",
        "<div><title>{{ title }} — {{ count }}</title></div>",
    ),
    ("title-entities", "<title>A &amp; B &lt; C &#169;</title>"),
    ("title-markup-text", "<title><b>not an element</b></title>"),
    (
        "title-multiline",
        "<title>\n  {{ title }}\n  text\n</title>",
    ),
    ("empty", ""),
    ("text-root", "hello"),
    ("text-escape", "a &lt;b&gt; &amp; 'c' \"d\""),
    ("template-literal-chars", "<p>`${x}` \\ $ {</p>"),
    (
        "named-entities",
        "<div>&nbsp;&copy;&times;&#39;&quot;&#x27;&#128;</div>",
    ),
    (
        "whitespace-condense",
        "<div>\n  <span> a </span>\n  <span>b</span>\n</div>",
    ),
    ("whitespace-between-inline", "<p><b>a</b> <i>b</i></p>"),
    ("dropped-comment", "<div>a<!-- c -->b</div>"),
    (
        "dropped-comment-spacing",
        "<div>a <!-- c --> b<!-- d --><span/></div>",
    ),
    ("single-root", "<div>hi</div>"),
    ("root-empty-element", "<div></div>"),
    ("root-void", "<input>"),
    (
        "root-static-attrs",
        r#"<div id="a" class="b c" data-x="1" hidden></div>"#,
    ),
    (
        "root-entity-attr",
        r#"<div title="a&amp;b &quot;q&quot;" alt='x"y'></div>"#,
    ),
    (
        "root-reserved-attrs",
        r#"<div ref="r" key="k" :key="kk" :ref="rr"></div>"#,
    ),
    ("root-dynamic", r#"<div :id="id" :title="title">x</div>"#),
    (
        "root-class-merge",
        r#"<div :class="[a, b]" class="x y"></div>"#,
    ),
    (
        "root-style-merge",
        r#"<div style="color:red" :style="{ top: t }"></div>"#,
    ),
    (
        "root-spread",
        r#"<div id="a" v-bind="obj" class="c"></div>"#,
    ),
    ("root-two-spreads", r#"<div v-bind="a" v-bind="b"></div>"#),
    ("root-padded-bind", r#"<div :title=" x "></div>"#),
    ("root-shorthand", r#"<div :id :aria-label></div>"#),
    (
        "root-event-dropped",
        r#"<button type="button" @click="go(1)">Go</button>"#,
    ),
    (
        "root-static-on-key",
        r#"<div onClick="x" :onClick="y"></div>"#,
    ),
    (
        "root-svg",
        r#"<svg viewBox="0 0 1 1"><path d="M0"/><circle :r="r"/></svg>"#,
    ),
    (
        "nested-interpolation",
        "<div><span>{{ foo }} bar</span><span>baz {{ qux }}</span></div>",
    ),
    ("root-interpolation-run", "foo {{ bar }} baz"),
    ("root-interpolation", "{{ msg }}"),
    ("root-fragment", "<div>a</div>\n<p>b</p>"),
    ("root-fragment-text", "  <div>a</div> text "),
    (
        "interpolation-expressions",
        "<p>{{ a + b }} {{ fn(c, 'd') }} {{ obj.x?.y }} {{ `t${u}` }}</p>",
    ),
    (
        "interpolation-globals",
        "<p>{{ Math.max(a, 1) }} {{ JSON.stringify(o) }} {{ $slots }}</p>",
    ),
    ("interpolation-arrow", "<p>{{ list.map(i => i * k) }}</p>"),
    (
        "interpolation-bindings",
        "<p>{{ title }} {{ count }} {{ state.x }} {{ label }} {{ format(1) }}</p>",
    ),
    (
        "inline-attrs",
        r#"<div><input disabled id="a&amp;b" :checked="c" :value="v"><img src="x.png" alt=""></div>"#,
    ),
    (
        "inline-class-style",
        r#"<div><p :class="k" class="s" style="color:red" :style="st"></p></div>"#,
    ),
    (
        "inline-class-no-static",
        r#"<div><p :class="{ on: active }"></p></div>"#,
    ),
    (
        "inline-static-class-valueless",
        r#"<div><p class :class="k"></p></div>"#,
    ),
    (
        "inline-order",
        r#"<div><p :style="s" v-bind="o" :class="c" class="x" style="y"></p></div>"#,
    ),
    ("inline-spread", r#"<div><p v-bind="obj" id="q"></p></div>"#),
    (
        "inline-reserved",
        r#"<div><p ref="r" key="k" :key="kk" :ref="rr" hidden></p></div>"#,
    ),
    (
        "inline-boolean",
        r#"<div><button :disabled="busy" :readonly="ro">b</button></div>"#,
    ),
    (
        "inline-padded",
        r#"<div><span :title=" x "></span><span :data-x="a &amp;&amp; b"></span></div>"#,
    ),
    (
        "inline-shorthand",
        r#"<div><span :title :data-id></span></div>"#,
    ),
    (
        "inline-events",
        r#"<div><a href="/x" @click.prevent="go" v-on="handlers">x</a></div>"#,
    ),
    (
        "inline-special-props",
        r#"<div><p :innerHTML="h" :textContent="t"></p></div>"#,
    ),
    (
        "inline-bindings",
        r#"<div><p :title="title" :class="cls" :data-n="count + 1"></p></div>"#,
    ),
    ("pre-text", "<pre>\n  x\n</pre>"),
    (
        "deep-nesting",
        "<main><section><article><header><h1>{{ t }}</h1></header><p>b</p></article></section></main>",
    ),
    (
        "table-explicit",
        "<table><tbody><tr><td>{{ a }}</td></tr></tbody></table>",
    ),
    ("table-implicit", "<table><tr><td>a</td></tr></table>"),
    (
        "table-implicit-tr",
        "<table><tbody><td>{{ a }}</td></tbody></table>",
    ),
    (
        "list-markup",
        "<ul><li>a</li><li :class=\"c\">{{ b }}</li></ul>",
    ),
    ("unicode", "<p title=\"日本\">こんにちは {{ name }} 🎉</p>"),
    ("template-plain", "<template><div>a</div></template>"),
    (
        "template-nested",
        "<div><template lang=\"x\"><b :title=\"t\">{{ v }}</b></template></div>",
    ),
    (
        "iframe",
        "<div><iframe :src=\"u\" frameborder=\"0\" allow=\"x\"></iframe></div>",
    ),
    (
        "iframe-content",
        "<div><iframe><p>fallback {{ f }}</p></iframe><noscript><img src=\"a.png\"></noscript></div>",
    ),
    (
        "legacy-content-tags",
        "<div><noembed>x</noembed><noframes>y</noframes><xmp>a &lt; b</xmp></div>",
    ),
    ("v-model-div", r#"<div v-model="msg"></div>"#),
    ("v-model-svg", r#"<svg v-model="msg"></svg>"#),
    ("v-model-math", r#"<math v-model="msg"></math>"#),
    ("v-model-template", r#"<template v-model="msg"></template>"#),
    (
        "v-model-nested",
        r#"<section><div v-model="msg"></div></section>"#,
    ),
    ("v-pre-text", "<div v-pre>{{ not }} an interpolation</div>"),
    (
        "v-pre-whitespace",
        "<pre>\n  a  b\n</pre><code v-pre class=\"font-code\">\n    {{ variable }}\n  </code>",
    ),
];

/// Templates the selector must keep on the legacy walker in this slice.
pub(super) const REFUSED: &[(&str, &str)] = &[
    (
        "dynamic-slot-name-expression",
        r#"<Foo><template #[names[0]]>x</template></Foo>"#,
    ),
    (
        "nested-slot-carrier",
        r#"<Foo><div v-if="a"><template #a>x</template></div></Foo>"#,
    ),
    ("slot-name-twice", r#"<div><slot name :name="n" /></div>"#),
    (
        "outlet-v-pre",
        "<slot v-pre>{{ not }} an interpolation</slot>",
    ),
    (
        "directive-complex-arg",
        r#"<div><p v-focus:[a+b]="x"></p></div>"#,
    ),
    (
        "dynamic-key-expression",
        r#"<div><p :[a+b]="val"></p></div>"#,
    ),
    ("v-model-argument", r#"<input v-model:foo="msg">"#),
    ("invalid-expression", "<div>{{ a &amp;&amp; b }}</div>"),
    ("invalid-bind", r#"<div :title="a +"></div>"#),
    ("script", "<div><script>var a = 1 < 2</script></div>"),
    ("style", "<div><style>.a > b { }</style></div>"),
    ("whitespace-entities", "<p>a&#10;&#32; b</p>"),
    (
        "component-model-dynamic-arg",
        r#"<div><Foo v-model:[prop]="value" /></div>"#,
    ),
];

/// TypeScript-only expressions: owned under `is_ts`, refused without it
/// (the shipped transform reports them as invalid JavaScript).
pub(super) const ADMITTED_TS: &[(&str, &str)] = &[
    (
        "interpolation-ts",
        "<p>{{ (value as string).trim() }} {{ n! + 1 }}</p>",
    ),
    (
        "inline-ts",
        r#"<div><p :title="(t as string)" :data-n="n satisfies number"></p></div>"#,
    ),
    (
        "root-ts",
        r#"<div :title="<string>x" :data-y="fn<T>(y)">{{ z as any }}</div>"#,
    ),
    (
        "component-model-ts",
        r#"<Bar v-model="(m as any)" /><Foo><Bar v-model=" (n as number) " /></Foo>"#,
    ),
];
