//! Control-flow, loop, and element-directive fixtures for the SSR emitter
//! differential.

/// Templates the string plan must own.
pub(super) const ADMITTED: &[(&str, &str)] = &[
    (
        "if-chain-root",
        r#"<div v-if="ok">a</div><p v-else-if="b">b</p><span v-else>c</span>"#,
    ),
    ("if-root-no-else", r#"<div v-if="ok">a</div>"#),
    ("if-nested-no-else", r#"<div><p v-if="x">1</p></div>"#),
    (
        "if-padded-condition",
        r#"<div><p v-if=" x &amp;&amp; y ">1</p></div>"#,
    ),
    (
        "if-template-fragment",
        r#"<template v-if="ok"><a>1</a><b>2</b></template>"#,
    ),
    (
        "if-template-single",
        r#"<template v-if="ok"><a>1</a></template><p v-else>x</p>"#,
    ),
    (
        "if-template-text",
        r#"<div><template v-if="ok">hi {{ name }}</template></div>"#,
    ),
    (
        "if-template-empty",
        r#"<div><template v-if="ok"></template></div>"#,
    ),
    (
        "if-with-text-siblings",
        "<div>before <p v-if=\"a\">x</p> after</div>",
    ),
    ("if-root-with-text", "hello <p v-if=\"a\">x</p>"),
    (
        "if-inside-if",
        r#"<div v-if="a"><p v-if="b">x</p><i v-else>y</i></div>"#,
    ),
    (
        "if-chain-sibling-chains",
        r#"<div><p v-if="a">1</p><p v-if="b">2</p><p v-else>3</p></div>"#,
    ),
    (
        "for-in-element",
        r#"<ul><li v-for="(item, i) in items" :key="item.id">{{ item.name }} {{ i }}</li></ul>"#,
    ),
    ("for-root", r#"<div v-for="item in items">{{ item }}</div>"#),
    (
        "for-destructure",
        r#"<template v-for="{ a, b } in list" :key="a"><i>{{ a }}</i><b>{{ b }}</b></template>"#,
    ),
    (
        "for-range-and-template",
        r#"<div><span v-for="n in 3">{{ n }}</span><template v-for="x in xs"><i>{{ x }}</i></template></div>"#,
    ),
    (
        "for-keyed-template-single",
        r#"<div><template v-for="x in xs" :key="x"><i>{{ x }}</i></template></div>"#,
    ),
    (
        "for-keyed-template-text",
        r#"<div><template v-for="x in xs" :key="x">{{ x }}</template></div>"#,
    ),
    (
        "for-object",
        r#"<dl><dt v-for="(value, key, index) in obj">{{ key }}={{ value }}#{{ index }}</dt></dl>"#,
    ),
    (
        "for-nested",
        r#"<div v-for="row in rows"><span v-for="cell in row.cells">{{ cell }} {{ row.id }}</span></div>"#,
    ),
    (
        "for-shadowing",
        r#"<div><p v-for="item in items" :title="item.title" :class="{ on: item.on }">{{ item }} {{ other }}</p></div>"#,
    ),
    (
        "for-destructure-default",
        r#"<ul><li v-for="({ id, label = 'x' }, idx) in list">{{ id }}{{ label }}{{ idx }}</li></ul>"#,
    ),
    (
        "for-of",
        r#"<ul><li v-for="item of items">{{ item }}</li></ul>"#,
    ),
    (
        "for-inside-if",
        r#"<div v-if="show"><p v-for="x in xs">{{ x }}</p></div>"#,
    ),
    (
        "if-inside-for",
        r#"<div><template v-for="x in xs"><p v-if="x.a">{{ x.a }}</p><i v-else>-</i></template></div>"#,
    ),
    (
        "if-on-for-item",
        r#"<ul><li v-for="x in xs"><b v-if="x">{{ x }}</b></li></ul>"#,
    ),
    ("show-root", r#"<div v-show="ok"></div>"#),
    (
        "show-inline",
        r#"<div><p v-show="ok"></p><p v-show="ok" style="a:b"></p><p v-show="ok" :style="s"></p><p :style="s" style="c:d" v-show="ok"></p></div>"#,
    ),
    (
        "show-padded",
        r#"<div><p v-show=" a &amp;&amp; b ">x</p></div>"#,
    ),
    (
        "show-root-style",
        r#"<div style="a:b" v-show="ok" :style="s"></div>"#,
    ),
    (
        "html-text",
        r#"<div><p v-html="h">ignored</p><p v-text="t"></p><p v-html="'&lt;b&gt;'"></p></div>"#,
    ),
    (
        "html-root",
        r#"<div v-html="raw" class="c"><span>ignored</span></div>"#,
    ),
    ("text-root", r#"<span v-text=" msg "></span>"#),
    (
        "model-inline",
        r#"<div><input v-model="msg"><input type="checkbox" v-model="c"><input type="radio" value="a" v-model="r"><input :type="t" v-model="v"><textarea v-model="txt"></textarea></div>"#,
    ),
    (
        "model-select",
        r#"<div><select v-model="s"><option value="a">A</option><option :value="b">B</option><option>C</option><optgroup label="g"><option value="d">D</option></optgroup></select></div>"#,
    ),
    (
        "model-select-for",
        r#"<div><select v-model="s"><option v-for="o in opts" :value="o.v">{{ o.l }}</option></select></div>"#,
    ),
    ("model-root-text", r#"<input v-model=" msg " class="x">"#),
    (
        "model-root-checkbox",
        r#"<input type="checkbox" v-model="c" value="z">"#,
    ),
    (
        "model-root-radio",
        r#"<input type="radio" v-model="c" value="z">"#,
    ),
    (
        "model-root-dynamic",
        r#"<input :type="t" v-model="v" id="q">"#,
    ),
    (
        "model-root-dynamic-spread",
        r#"<input v-bind="attrs" :type="t" v-model="v">"#,
    ),
    (
        "model-root-textarea",
        r#"<textarea v-model="t"></textarea>"#,
    ),
    (
        "model-root-select",
        r#"<select v-model="s"><option value="a">A</option></select>"#,
    ),
    (
        "model-modifiers",
        r#"<div><input v-model.trim="a"><input v-model.number="b" type="number"></div>"#,
    ),
    (
        "model-in-for",
        r#"<div><input v-for="item in items" v-model="item.value"></div>"#,
    ),
    (
        "textarea-plain",
        "<div><textarea>a &amp; b</textarea></div>",
    ),
    (
        "select-plain",
        "<select><option value=\"a\">A</option></select>",
    ),
    (
        "if-gap-comment",
        "<div>\n  <p v-if=\"a\">1</p>\n  <!-- between -->\n  <p v-else-if=\"b\">2</p>\n\n  <p v-else>3</p>\n</div>",
    ),
    (
        "if-gap-root",
        "<p v-if=\"a\">1</p>\n<!-- x -->\n<Foo v-else />",
    ),
];
