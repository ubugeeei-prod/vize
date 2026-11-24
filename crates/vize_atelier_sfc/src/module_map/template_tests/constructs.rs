//! One of each construct a template expression can sit in, in each module
//! shape: every segment that lands in the template is pinned.
//!
//! A slot's parameters carry no segment of their own in any shape: the
//! `_withCtx(` before them maps to the slot's `<template>`.

use super::{CONSTRUCTS, INLINE, SEPARATE, SSR, Shape, compile, template_segments};

#[test]
fn inline_typescript_maps_every_template_construct() {
    let shape = Shape {
        typescript: true,
        ..INLINE
    };
    let result = compile(CONSTRUCTS, shape);
    assert_eq!(
        template_segments(&result, CONSTRUCTS),
        [
            r#"19:9 "(_openBlock(" -> 13:2 "<section :ti""#,
            r#"19:45 "section\", {⏎" -> 13:3 "section :tit""#,
            r#"20:4 "title: title" -> 13:12 "title=\"title""#,
            r#"20:11 "title.value," -> 13:19 "title\" :clas""#,
            r#"21:4 "class: _norm" -> 13:27 "class=\"{ on:""#,
            r#"21:27 "{ on: open.v" -> 13:34 "{ on: open }""#,
            r#"23:5 "open.value)⏎" -> 14:13 "open\">{{ tit""#,
            r#"24:8 "(_openBlock(" -> 14:4 "<p v-if=\"ope""#,
            r#"24:44 "p\", { key: 0" -> 14:5 "p v-if=\"open""#,
            r#"24:77 "title.value)" -> 14:22 "title }}</p>""#,
            r#"25:8 "(_openBlock(" -> 15:4 "<span v-else""#,
            r#"25:44 "span\", { key" -> 15:5 "span v-else>""#,
            r#"25:64 "closed\")),⏎ " -> 15:17 "closed</span""#,
            r#"26:25 "ul\", null, [" -> 16:5 "ul>⏎      <l""#,
            r#"27:6 "(_openBlock(" -> 17:6 "<li v-for=\"(""#,
            r#"27:74 "items.value," -> 17:34 "items\" :key=""#,
            r#"27:88 "item, index)" -> 17:18 "item, index)""#,
            r#"27:94 "index) => {⏎" -> 17:24 "index) in it""#,
            r#"28:51 "li\", {⏎     " -> 17:7 "li v-for=\"(i""#,
            r#"29:15 "item.id,⏎   " -> 17:47 "item.id\" @cl""#,
            r#"30:10 "onClick: ($e" -> 17:57 "click=\"pick(""#,
            r#"30:19 "($event: any" -> 17:64 "pick(item.id""#,
            r#"31:28 "index) + \": " -> 18:11 "index }}: {{""#,
            r#"31:38 ": \" + _toDis" -> 18:19 ": {{ item.na""#,
            r#"31:61 "item.name), " -> 18:24 "item.name }}""#,
            r#"34:41 "input\", {⏎  " -> 21:5 "input v-mode""#,
            r#"35:7 "onUpdate:mod" -> 21:11 "v-model=\"tex""#,
            r#"35:55 "($event: any" -> 21:11 "v-model=\"tex""#,
            r#"37:20 "text.value]⏎" -> 21:20 "text\" />⏎   ""#,
            r#"39:17 "Child, {⏎   " -> 22:5 "Child :label""#,
            r#"40:6 "label: title" -> 22:12 "label=\"title""#,
            r#"40:13 "title.value," -> 22:19 "title\" @done""#,
            r#"41:6 "onDone: _cac" -> 22:27 "done=\"open =""#,
            r#"41:40 "($event: any" -> 22:33 "open = false""#,
            r#"43:6 "default: _wi" -> 23:17 "default=\"{ r""#,
            r#"43:15 "_withCtx(({ " -> 23:6 "<template #d""#,
            r#"44:42 "row.name), 1" -> 23:38 "row.name }}<""#,
        ],
        "{}",
        result.code
    );
}

#[test]
fn a_separate_javascript_render_maps_every_template_construct() {
    let source = CONSTRUCTS
        .replace(" lang=\"ts\"", "")
        .replace("id: number", "id");
    let result = compile(&source, SEPARATE);
    assert_eq!(
        template_segments(&result, &source),
        [
            r#"4:0 "function _sf" -> 12:10 "⏎  <section ""#,
            r#"5:9 "(_openBlock(" -> 13:2 "<section :ti""#,
            r#"5:45 "section\", {⏎" -> 13:3 "section :tit""#,
            r#"6:4 "title: $setu" -> 13:12 "title=\"title""#,
            r#"6:11 "$setup.title" -> 13:19 "title\" :clas""#,
            r#"7:4 "class: _norm" -> 13:27 "class=\"{ on:""#,
            r#"7:27 "{ on: $setup" -> 13:34 "{ on: open }""#,
            r#"7:33 "$setup.open " -> 13:40 "open }\">⏎   ""#,
            r#"9:5 "$setup.open)" -> 14:13 "open\">{{ tit""#,
            r#"10:8 "(_openBlock(" -> 14:4 "<p v-if=\"ope""#,
            r#"10:44 "p\", { key: 0" -> 14:5 "p v-if=\"open""#,
            r#"10:77 "$setup.title" -> 14:22 "title }}</p>""#,
            r#"11:8 "(_openBlock(" -> 15:4 "<span v-else""#,
            r#"11:44 "span\", { key" -> 15:5 "span v-else>""#,
            r#"11:64 "closed\")),⏎ " -> 15:17 "closed</span""#,
            r#"12:25 "ul\", null, [" -> 16:5 "ul>⏎      <l""#,
            r#"13:6 "(_openBlock(" -> 17:6 "<li v-for=\"(""#,
            r#"13:74 "$setup.items" -> 17:34 "items\" :key=""#,
            r#"13:89 "item, index)" -> 17:18 "item, index)""#,
            r#"13:95 "index) => {⏎" -> 17:24 "index) in it""#,
            r#"14:51 "li\", {⏎     " -> 17:7 "li v-for=\"(i""#,
            r#"15:15 "item.id,⏎   " -> 17:47 "item.id\" @cl""#,
            r#"16:10 "onClick: $ev" -> 17:57 "click=\"pick(""#,
            r#"16:19 "$event => ($" -> 17:64 "pick(item.id""#,
            r#"17:28 "index) + \": " -> 18:11 "index }}: {{""#,
            r#"17:38 ": \" + _toDis" -> 18:19 ": {{ item.na""#,
            r#"17:61 "item.name), " -> 18:24 "item.name }}""#,
            r#"20:41 "input\", {⏎  " -> 21:5 "input v-mode""#,
            r#"21:7 "onUpdate:mod" -> 21:11 "v-model=\"tex""#,
            r#"21:29 "$event => ((" -> 21:11 "v-model=\"tex""#,
            r#"23:20 "$setup.text]" -> 21:20 "text\" />⏎   ""#,
            r#"25:17 "$setup[\"Chil" -> 22:5 "Child :label""#,
            r#"26:6 "label: $setu" -> 22:12 "label=\"title""#,
            r#"26:13 "$setup.title" -> 22:19 "title\" @done""#,
            r#"27:6 "onDone: $eve" -> 22:27 "done=\"open =""#,
            r#"27:14 "$event => ($" -> 22:33 "open = false""#,
            r#"29:6 "default: _wi" -> 23:17 "default=\"{ r""#,
            r#"29:15 "_withCtx(({ " -> 23:6 "<template #d""#,
            r#"30:42 "row.name), 1" -> 23:38 "row.name }}<""#,
        ],
        "{}",
        result.code
    );
}

#[test]
fn a_server_render_maps_every_template_construct() {
    // The server render writes a component's prop bag, a `v-model` attribute
    // and a slot's parameters as plain text: they have no segment of their
    // own and resolve to the component, element or slot anchor before them.
    let result = compile(CONSTRUCTS, SSR);
    assert_eq!(
        template_segments(&result, CONSTRUCTS),
        [
            r#"5:0 "function ssr" -> 12:10 "⏎  <section ""#,
            r#"6:10 "section${_ss" -> 13:3 "section :tit""#,
            r#"6:49 "title: $setu" -> 13:12 "title=\"title""#,
            r#"6:56 "$setup.title" -> 13:19 "title\" :clas""#,
            r#"6:70 "class: { on:" -> 13:27 "class=\"{ on:""#,
            r#"6:77 "{ on: $setup" -> 13:34 "{ on: open }""#,
            r#"6:83 "$setup.open " -> 13:40 "open }\">⏎   ""#,
            r#"7:2 "if ($setup.o" -> 14:4 "<p v-if=\"ope""#,
            r#"7:6 "$setup.open)" -> 14:13 "open\">{{ tit""#,
            r#"8:12 "p>${_ssrInte" -> 14:5 "p v-if=\"open""#,
            r#"8:32 "$setup.title" -> 14:22 "title }}</p>""#,
            r#"9:4 "else {⏎    _" -> 15:4 "<span v-else""#,
            r#"10:12 "span>closed<" -> 15:5 "span v-else>""#,
            r#"10:17 "closed</span" -> 15:17 "closed</span""#,
            r#"12:10 "ul>`)⏎  _pus" -> 16:5 "ul>⏎      <l""#,
            r#"14:2 "_ssrRenderLi" -> 17:6 "<li v-for=\"(""#,
            r#"14:17 "$setup.items" -> 17:34 "items\" :key=""#,
            r#"14:32 "item, index)" -> 17:18 "item, index)""#,
            r#"14:38 "index) => {⏎" -> 17:24 "index) in it""#,
            r#"15:12 "li>${_ssrInt" -> 17:7 "li v-for=\"(i""#,
            r#"15:33 "index)}: ${_" -> 18:11 "index }}: {{""#,
            r#"15:40 ": ${_ssrInte" -> 18:19 ": {{ item.na""#,
            r#"15:60 "item.name)}<" -> 18:24 "item.name }}""#,
            r#"18:15 "input${_ssrR" -> 21:5 "input v-mode""#,
            r#"19:28 "$setup.Child" -> 22:5 "Child :label""#,
            r#"20:4 "default: _wi" -> 23:17 "default=\"{ r""#,
            r#"20:13 "_withCtx(({ " -> 23:6 "<template #d""#,
            r#"22:33 "row.name)}`)" -> 23:38 "row.name }}<""#,
        ],
        "{}",
        result.code
    );
}
