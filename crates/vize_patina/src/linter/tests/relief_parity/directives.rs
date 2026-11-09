//! The directive-spelling rules moved off the Relief visitor.

use crate::rule::Rule;
use crate::rules::opinionated::vue::NoVText;
use crate::rules::vue::{
    NoCustomModifiersOnVModel, NoDeprecatedVBindSync, NoDeprecatedVIs,
    NoDeprecatedVOnNativeModifier, NoDeprecatedVOnNumberModifiers, ValidVCloak, ValidVHtml,
    ValidVOnce, ValidVShow, ValidVText,
};

fn family() -> Vec<Box<dyn Rule>> {
    vec![
        Box::new(ValidVCloak),
        Box::new(ValidVOnce),
        Box::new(ValidVText),
        Box::new(ValidVHtml),
        Box::new(ValidVShow),
        Box::new(NoVText),
    ]
}

const TEMPLATES: &[&str] = &[
    r#"<div v-cloak></div><div v-cloak="foo"></div><div v-cloak:foo></div><div v-cloak.foo></div>"#,
    r#"<div v-cloak=""></div><div v-cloak="&#32;"></div><div v-cloak="&amp;"></div><div v-cloak=></div>"#,
    r#"<div v-cloak:[x].a.b="y"></div><div v-cloak:></div><div v-cloak.></div><div v-cloak..a></div>"#,
    r#"<div v-once></div><div v-once="x"></div><p v-once:a.b.c="d"></p><li v-for="i in xs" v-once></li>"#,
    r#"<div v-text></div><div v-text=""></div><div v-text="msg"></div><div v-text:arg.mod="foo"></div>"#,
    r#"<div v-text="&nbsp;"></div><div v-text="  &#9; "></div><div v-text="&lt;"></div><b v-text=日本語😀></b>"#,
    r#"<div v-html></div><div v-html="raw"></div><div v-html:arg="raw"></div><div v-html.trim="raw"></div>"#,
    r#"<MyComp v-html="raw" v-text="t"/><template v-text="t"><i/></template><textarea v-text></textarea>"#,
    r#"<div v-show></div><div v-show=""></div><div v-show="ok"></div><div v-show:arg.a.b="ok"></div>"#,
    r#"<template v-show="ok"><div/></template><Template v-show="ok"><div/></Template><div v-show:[d].x="ok"></div>"#,
    r#"<div v-if="a" v-show="b" v-text="c"></div><div v-else v-html="d" v-cloak></div>"#,
    r#"<div v-pre><span v-text="x" v-show v-cloak="y"></span></div><pre v-pre><b v-html></b></pre><p v-show></p>"#,
    r#"<svg><text v-text="a" v-show></text></svg><table><tr v-show="x"><td v-html></td></tr></table>"#,
    r#"<div V-TEXT="a" v-text.prop="c" .text="d" :text="e" @text="f"></div><div v-text="a" v-text="b"></div>"#,
    "<div\n  v-show\n  v-text:a.b\n  v-cloak=\"z\"\n></div>",
];

const JSX: &[&str] = &[
    r#"const A = () => <div v-show={visible} v-html={raw} v-text={msg} />;"#,
    r#"const A = () => <div v-show v-cloak v-once v-text />;"#,
    r#"const A = () => <><div v-show:arg={x} /><div v-text="literal" /><template v-show={x} /></>;"#,
];

#[test]
fn directive_shape_rules_match_their_relief_hooks() {
    super::assert_parity(family, TEMPLATES, JSX);
}

fn deprecated_family() -> Vec<Box<dyn Rule>> {
    vec![
        Box::new(NoDeprecatedVBindSync),
        Box::new(NoDeprecatedVOnNativeModifier),
        Box::new(NoDeprecatedVOnNumberModifiers),
        Box::new(NoDeprecatedVIs),
        Box::new(NoCustomModifiersOnVModel),
    ]
}

const DEPRECATED_TEMPLATES: &[&str] = &[
    r#"<C :foo.sync="x" v-bind:bar.sync="y" :baz="z" .qux.sync="w" :[d].sync="v" v-bind.sync="o"/>"#,
    r#"<C @click.native="f" v-on:input.stop.native.native="g" @[e].native="h" @click.native.prevent="i"/>"#,
    r#"<input @keyup.13="a" @keydown.enter.27="b" @keyup.0.13="c" @keyup.13suffix="d" v-on:keyup.112="e">"#,
    r#"<input @keyup.40.13="a" @keyup.-1="b" @keyup.1e3="c" @keyup.0x10="d" @keyup.9007199254740993="e">"#,
    r#"<div v-is="'c'"></div><tr v-is:arg.mod="'row'"></tr><div v-is></div><div v-is.></div><div v-is:[x]="y"></div>"#,
    r#"<C v-model.foo="x"/><C v-model.trim.lazy.number.bar="y"/><input v-model.foo="z"><div is="vue:c" v-model.baz="w"/>"#,
    r#"<component :is="c" v-model:title.cap="t"/><MyInput v-model.a.b="v"/><select v-model.x="s"></select>"#,
    r#"<div v-pre><C :a.sync="x" @b.native="y" @keyup.13="z" v-is="w"/></div><C v-if="ok" :a.sync="x"/>"#,
];

const DEPRECATED_JSX: &[&str] = &[
    r#"const A = () => <C onClick={f} v-model={x} v-is={y} />;"#,
    r#"const A = () => <><C v-model:foo={x} /><input onKeyup={f} /></>;"#,
];

#[test]
fn deprecated_modifier_rules_match_their_relief_hooks() {
    super::assert_parity(deprecated_family, DEPRECATED_TEMPLATES, DEPRECATED_JSX);
}
