//! The accessibility rules moved off the Relief visitor.

use crate::rule::Rule;
use crate::rules::a11y::{
    AltText, AnchorHasContent, AnchorIsValid, AriaProps, AriaRole, AriaUnsupportedElements,
    ClickEventsHaveKeyEvents, FormControlHasLabel, HeadingHasContent, IframeHasTitle, ImgAlt,
    InteractiveSupportsFocus, LabelHasFor, MediaHasCaption, MouseEventsHaveKeyEvents, NoAccessKey,
    NoAriaHiddenOnFocusable, NoAutofocus, NoDistractingElements, NoIForIcon, NoRedundantRoles,
    NoReferToNonExistentId, NoRolePresentationOnFocusable, NoStaticElementInteractions,
    RoleHasRequiredAriaProps, TabindexNoPositive,
};
use crate::rules::html::{
    DeprecatedAttr, DeprecatedElement, NoConsecutiveBr, NoDuplicateDt, RequireDatetime,
};
use crate::rules::opinionated::a11y::{HeadingLevels, PlaceholderLabelOption, UseList};
use crate::rules::opinionated::html::{NoDupeStyleProperties, NoDuplicateClass};
use crate::rules::opinionated::vue::{HtmlButtonHasType, NoBooleanAttrValue, NoInlineStyle};
use crate::rules::vapor::{NoVueLifecycleEvents, PreferStaticClass};
use crate::rules::vue::{
    NoBareStringsInTemplate, NoInvalidHtmlAttribute, NoMultiSpaces, NoStaticInlineStyles,
    NoTemplateTargetBlank, NoTextareaMustache, NoUnsandboxedIframe, PermittedContents,
    RequireVForKey,
};

fn family() -> Vec<Box<dyn Rule>> {
    vec![
        Box::new(AltText),
        Box::new(AnchorHasContent),
        Box::new(AriaProps),
        Box::new(AriaRole::default()),
        Box::new(AriaRole {
            ignore_non_dom: true,
        }),
        Box::new(AriaUnsupportedElements),
        Box::new(ClickEventsHaveKeyEvents),
        Box::new(LabelHasFor),
        Box::new(NoReferToNonExistentId),
        Box::new(NoStaticElementInteractions),
        Box::new(RoleHasRequiredAriaProps),
    ]
}

fn existing_family() -> Vec<Box<dyn Rule>> {
    vec![
        Box::new(AnchorIsValid),
        Box::new(FormControlHasLabel),
        Box::new(HeadingHasContent),
        Box::new(IframeHasTitle),
        Box::new(ImgAlt),
        Box::new(InteractiveSupportsFocus),
        Box::new(MediaHasCaption),
        Box::new(MouseEventsHaveKeyEvents),
        Box::new(NoAccessKey),
        Box::new(NoAriaHiddenOnFocusable),
        Box::new(NoAutofocus),
        Box::new(NoDistractingElements),
        Box::new(NoIForIcon),
        Box::new(NoRedundantRoles),
        Box::new(NoRolePresentationOnFocusable),
        Box::new(PlaceholderLabelOption),
        Box::new(TabindexNoPositive),
    ]
}

fn html_family() -> Vec<Box<dyn Rule>> {
    vec![
        Box::new(DeprecatedAttr),
        Box::new(DeprecatedElement),
        Box::new(NoConsecutiveBr),
        Box::new(NoDuplicateDt),
        Box::new(RequireDatetime),
    ]
}

fn vapor_family() -> Vec<Box<dyn Rule>> {
    vec![Box::new(NoVueLifecycleEvents), Box::new(PreferStaticClass)]
}

fn remaining_facade_family() -> Vec<Box<dyn Rule>> {
    vec![
        Box::new(HeadingLevels),
        Box::new(UseList),
        Box::new(NoDupeStyleProperties),
        Box::new(NoDuplicateClass),
        Box::new(HtmlButtonHasType::default()),
        Box::new(NoBooleanAttrValue),
        Box::new(NoInlineStyle),
        Box::new(NoBareStringsInTemplate),
        Box::new(NoInvalidHtmlAttribute),
        Box::new(NoMultiSpaces::default()),
        Box::new(NoStaticInlineStyles),
        Box::new(NoTemplateTargetBlank),
        Box::new(NoTextareaMustache),
        Box::new(NoUnsandboxedIframe),
        Box::new(PermittedContents),
        Box::new(RequireVForKey),
    ]
}

const REMAINING_FACADE_TEMPLATES: &[&str] = &[
    r#"Root-level text<div>Nested text</div>"#,
    r#"<h1>Title</h1><h3>Skipped</h3><p>- Item</p>"#,
    r#"<div class="btn btn" style="color:red;color:blue">Visible text</div>"#,
    r#"<button  disabled="disabled">Submit</button><a rel="friend">Link</a>"#,
    r#"<a href="https://example.com" target="_blank">External</a>"#,
    r#"<textarea>{{ message }}</textarea><iframe src="/embed"></iframe>"#,
    r#"<p><div>browser repair</div></p>"#,
    r#"<template v-for="item in items"><span>{{ item }}</span></template><li v-for="item in items">{{ item }}</li>"#,
];

const REMAINING_FACADE_JSX: &[&str] = &[
    r#"const A = () => <>{'Expression text'}<div>Nested text</div></>;"#,
    r#"const A = () => <><h1>Title</h1><h3>Skipped</h3><p>- Item</p></>;"#,
    r#"const A = () => <div className="btn btn" style="color:red;color:blue">Visible text</div>;"#,
    r#"const A = () => <><button disabled="disabled">Submit</button><a rel="friend">Link</a></>;"#,
    r#"const A = () => <a href="https://example.com" target="_blank">External</a>;"#,
    r#"const A = () => <><textarea>{message}</textarea><iframe src="/embed" /></>;"#,
    r#"const A = () => <p><div>nested block</div></p>;"#,
    r#"const A = () => items.map((item) => <li>{item}</li>);"#,
];

const TEMPLATES: &[&str] = &[
    r#"<button role="button">Redundant</button><button>Native</button><div role="button">Explicit</div>"#,
    r#"<select required><option value="jp">Japan</option></select><select required><option value="">Choose</option><option value="jp">Japan</option></select>"#,
    r#"<div @vue:mounted="f"></div><div @click="f"></div><div :class="'static'"></div><div class="static"></div><div :class="dynamic"></div>"#,
    // The HTML facade's element-only rules use the same traversal contract.
    r#"<div align="center">Text</div><div style="text-align:center">Text</div><font>Old</font><span>New</span>"#,
    r#"<div><br><br></div><div><br><span>Text</span><br></div>"#,
    r#"<dl><dt>Term</dt><dt>Term</dt><dd>Definition</dd></dl><dl><dt>One</dt><dt>Two</dt><dd>Definition</dd></dl>"#,
    r#"<time>2026</time><time datetime="2026-11-05">2026</time>"#,
    // Existing facade rules keep their complete diagnostics with narrower hooks.
    r#"<iframe></iframe><iframe title="Map"></iframe><h1></h1><h1>Title</h1>"#,
    r#"<label for="known">Name</label><input id="known"><input id="missing"><label><input></label>"#,
    r#"<a href="">Empty</a><a href="/">Home</a><img src="a"><img src="a" alt="Photo">"#,
    r#"<button accesskey="k" autofocus tabindex="1">Go</button><button tabindex="0">Go</button>"#,
    r#"<marquee>Moving</marquee><blink>Blinking</blink><i class="fas fa-save"></i><span class="fas fa-save"></span>"#,
    r#"<div role="button"></div><div role="button" tabindex="0"></div><div role="presentation" tabindex="0"></div>"#,
    r#"<div aria-hidden="true"><button>Hidden</button></div><button aria-hidden="true">Hidden</button><button>Visible</button>"#,
    r#"<video controls></video><video controls><track kind="captions"></video><audio></audio>"#,
    r#"<div @mouseover="f"></div><div @mouseover="f" @focus="f"></div><div @mouseout="f" @blur="f"></div>"#,
    // Table recovery keeps this v-pre subtree on both routes. The implied-end
    // shapes where Relief and Vue differ have their own Vue-oracle test below.
    "<table><td v-pre><img src=\"a\"><img :alt=\"x\"><div @click=\"f\"></div><span @click=\"f\" @keydown=\"g\"></span><a href=\"/\"></a><label>x</label><div role=\"checkbox\"></div><div :aria-labeledby=\"x\"></div><div :role=\"&apos;buton&apos;\"></div><meta :role=\"x\"><label for=\"nope\">y</label></td></table>",
    // Truncated and unbalanced input, as an editor sends while typing.
    r#"<div @click="f""#,
    r#"<label for="x"><div role="checkbox""#,
    r#"<a href="/"><span>"#,
    r#"<div></span><img src="a"></p><label>x"#,
    "<img alt=",
    // alt-text
    r#"<img src="a"><img alt="x"><img :alt="x"><img :[alt]="x"><IMG src="a"></IMG>"#,
    r#"<area href="/"><area aria-label="x"><area :aria-labelledby="x"><Comp is="img"/>"#,
    r#"<input type="image"><input type="image" alt="Go"><input type="text"><input :type="'image'">"#,
    r#"<object data="x"></object><object title="t"></object><object :aria-label="l"></object>"#,
    // anchor-has-content
    r#"<a href="/"></a><a href="/">text</a><a>{{ label }}</a><a> &nbsp; </a><a>&#32;</a>"#,
    r#"<a><img alt="x"></a><a><img alt=""></a><a><img :alt=""></a><a><img :alt="''"></a>"#,
    r#"<a><img :alt></a><a><img :alt="`x`"></a><a><img :[alt]="x"></a><a><img alt></a>"#,
    r#"<a aria-label="x"></a><a :aria-labelledby="id"></a><a><span>t</span></a><a><slot/></a>"#,
    r#"<a><template v-if="c">text</template></a><a v-if="x"></a><a v-for="i in n">{{ i }}</a>"#,
    r#"<a><span v-pre>{{ raw }}</span></a><a><!-- c --></a><a><svg><title>t</title></svg></a>"#,
    // aria-props
    r#"<div aria-labeledby="x" aria-label="y" :aria-hiiden="z" :[aria-foo]="w"></div>"#,
    r#"<div :aria-foo.camel="x" v-bind:aria-bar.prop="y" aria-busy aria-Label="z"></div>"#,
    r#"<div v-pre aria-labeledby="x" :aria-lable="y"></div><Comp aria-describeby="x"/>"#,
    // aria-role
    r#"<div role="buton"></div><div role="button"></div><div role="widget"></div><div role=""></div>"#,
    r#"<div :role="'buton'"></div><div :role="`modal`"></div><div :role="r"></div><div role></div>"#,
    r#"<div role="button  foo Dropdown"></div><Comp role="nope"/><my-el role="nope"></my-el>"#,
    r#"<div :role="'a' + 'b'"></div><div :[role]="'buton'"></div><div v-bind:role="'link'"></div>"#,
    // aria-unsupported-elements
    r#"<meta aria-hidden="true"><script role="x"></script><style :aria-label="x"></style>"#,
    r#"<html role="x"></html><Meta aria-hidden="true"/><meta :[aria-x]="y"><meta lang="en">"#,
    r#"<table><html role></html></table><table><tr><meta aria-x="1"></tr></table>"#,
    // click-events-have-key-events
    r#"<div @click="f"></div><div @click="f" @keydown="g"></div><button @click="f"></button>"#,
    r#"<div role="button" @click="f"></div><div role="listbox" @click="f"></div><div role role="button" @click="f"></div>"#,
    r#"<div aria-hidden="true" @click="f"></div><div :aria-hidden="true" @click="f"></div>"#,
    r#"<div role="presentation" @click="f"></div><my-el @click="f"></my-el><Comp @click="f"/>"#,
    r#"<div @[click]="f"></div><div v-on:click.stop="f"></div><div @click="f" @keyup.enter="g"></div>"#,
    // label-has-for
    r#"<label>Name</label><label for="x">Name</label><label htmlFor="x">N</label><label :for="id">N</label>"#,
    r#"<label v-bind="attrs">Name</label><label :[for]="x">N</label><label v-on="h">N</label>"#,
    r#"<label><input></label><label><span><select></select></span></label><label><Input/></label>"#,
    r#"<label><template v-for="i in n"><input></template></label><label><textarea v-if="x"></textarea></label>"#,
    // no-refer-to-non-existent-id
    r#"<label for="missing">x</label><input id="name"><div aria-labelledby="name other"></div>"#,
    r#"<div aria-describedby="a b"><span id="a"></span></div><label :for="x">y</label>"#,
    // no-static-element-interactions
    r#"<div @click="f"></div><span @mousedown="f"></span><div :role="r" @click="f"></div>"#,
    r#"<dialog @click="f"></dialog><div role="toolbar" @click="f"></div><div role="tab" @click="f"></div>"#,
    r#"<div @focus="f"></div><section @dblclick="f"></section><a @click="f"></a><div @[keydown]="f"></div>"#,
    // role-has-required-aria-props
    r#"<div role="checkbox"></div><div role="checkbox" aria-checked="false"></div><div role="slider"></div>"#,
    r#"<input type="checkbox" role="checkbox"><input :type="'radio'" role="radio"><div :role="'combobox'"></div>"#,
    r#"<div role="separator"></div><div role="separator" tabindex="0"></div><hr role="separator" :aria-valuenow="n">"#,
    r#"<div role="separator" contenteditable></div><div role="separator" contenteditable="false"></div>"#,
    r#"<a role="separator" href="/"></a><Comp role="checkbox"/><div role="heading" :[aria-level]="l"></div>"#,
];

const JSX: &[&str] = &[
    r#"const A = () => <div><button role="button">Redundant</button><button>Native</button><div role="button">Explicit</div></div>;"#,
    r#"const A = () => <div><select required><option value="jp">Japan</option></select><select required><option value="">Choose</option><option value="jp">Japan</option></select></div>;"#,
    r#"const A = () => <div><div class={'static'} /><div class="static" /><div class={dynamic} /></div>;"#,
    r#"const A = () => <div><div align="center">Text</div><div style={{textAlign: 'center'}}>Text</div><font>Old</font><span>New</span></div>;"#,
    r#"const A = () => <div><div><br /><br /></div><div><br /><span>Text</span><br /></div></div>;"#,
    r#"const A = () => <div><dl><dt>Term</dt><dt>Term</dt><dd>Definition</dd></dl><dl><dt>One</dt><dt>Two</dt><dd>Definition</dd></dl></div>;"#,
    r#"const A = () => <div><time>2026</time><time dateTime="2026-11-05">2026</time></div>;"#,
    r#"const A = () => <div><iframe /><iframe title="Map" /><h1 /><h1>Title</h1></div>;"#,
    r#"const A = () => <div><label htmlFor="known">Name</label><input id="known" /><input id="missing" /><label><input /></label></div>;"#,
    r#"const A = () => <div><a href="">Empty</a><a href="/">Home</a><img src="a" /><img src="a" alt="Photo" /></div>;"#,
    r#"const A = () => <div><button accesskey="k" autofocus tabindex="1">Go</button><button tabIndex={0}>Go</button></div>;"#,
    r#"const A = () => <div><marquee>Moving</marquee><blink>Blinking</blink><i className="fas fa-save" /><span className="fas fa-save" /></div>;"#,
    r#"const A = () => <div><div role="button" /><div role="button" tabIndex={0} /><div role="presentation" tabIndex={0} /></div>;"#,
    r#"const A = () => <div><div aria-hidden="true"><button>Hidden</button></div><button aria-hidden="true">Hidden</button><button>Visible</button></div>;"#,
    r#"const A = () => <div><video controls /><video controls><track kind="captions" /></video><audio /></div>;"#,
    r#"const A = () => <div><div onMouseOver={f} /><div onMouseOver={f} onFocus={f} /><div onMouseOut={f} onBlur={f} /></div>;"#,
    r#"const A = () => <div><img src="a" /><img alt="x" /><img alt={x} /><Img /><Foo.img /></div>;"#,
    r#"const A = () => <div><area href="/" /><area aria-label="x" /><input type="image" /><input type="image" alt="Go" /><object data="x" /><object title="t" /></div>;"#,
    r#"const A = () => <div><a href="/"></a><a href="/">text</a><a>{label}</a><a><img alt="x" /></a><a><img alt="" /></a><a><img alt={""} /></a></div>;"#,
    r#"const A = () => <div><a aria-label="x" /><a><span>t</span></a><a>{cond && <span>t</span>}</a><a>{items.map((i) => <span>{i}</span>)}</a><a><slot /></a></div>;"#,
    r#"const A = () => <div aria-labeledby="x" aria-label="y" aria-foo={x} aria-hiiden="true" />;"#,
    r#"const A = () => <div><div role="buton" /><div role="button" /><div role="widget" /><div role={"buton"} /><div role={`buton`} /><div role={r} /><div role="button  foo" /></div>;"#,
    r#"const A = () => <div><meta aria-hidden="true" /><script role="x" /><style aria-label={x} /></div>;"#,
    r#"const A = () => <div><div onClick={f} /><div onClick={f} onKeyDown={g} /><button onClick={f} /><div role="button" onClick={f} /></div>;"#,
    r#"const A = () => <div><div aria-hidden="true" onClick={f} /><div role="presentation" onClick={f} /><my-el onClick={f} /><Comp onClick={f} /></div>;"#,
    r#"const A = () => <div><label>Name</label><label htmlFor="x">Name</label><label for="x" /><label {...props}>Name</label></div>;"#,
    r#"const A = () => <div><label><input /></label><label>{cond && <input />}</label><label>{items.map((i) => <input />)}</label><label><span><select /></span></label></div>;"#,
    r#"const A = () => <div><label>{<input />}</label><label>{<select />}</label><label>{<textarea />}</label><label>{<><input /></>}</label></div>;"#,
    r#"const A = () => <div><label>{<span />}</label><label>{<><span /></>}</label><label>{<Input />}</label></div>;"#,
    r#"const A = () => <div><label>{cond && <span />}</label><label>{cond ? <input /> : <span />}</label><label>{a && (b ? <select /> : null)}</label><label>{rows.map((r) => <div><textarea /></div>)}</label></div>;"#,
    r#"const A = () => <div><label htmlFor="missing">x</label><div aria-labelledby="a b" /><span id="a" /></div>;"#,
    r#"const A = () => <div><span onMouseDown={f} /><div role={r} onClick={f} /><dialog onClick={f} /><div role="toolbar" onClick={f} /></div>;"#,
    r#"const A = () => <div><div role="checkbox" /><div role="checkbox" aria-checked="false" /><input type="checkbox" role="checkbox" /><div role="slider" /></div>;"#,
    r#"const A = () => <div><div role="separator" /><div role="separator" tabIndex={0} /><div role={"checkbox"} /><a role="separator" href="/" /></div>;"#,
    r#"const A = () => cond ? <div onClick={f} role="checkbox" /> : <img src="a" />;"#,
];

#[test]
fn a11y_rules_match_their_relief_hooks() {
    super::assert_parity(family, TEMPLATES, JSX);
}

#[test]
fn existing_a11y_traversals_preserve_complete_diagnostics() {
    super::assert_traversal_parity(existing_family, TEMPLATES, JSX);
}

#[test]
fn existing_html_traversals_preserve_complete_diagnostics() {
    super::assert_traversal_parity(html_family, TEMPLATES, JSX);
}

#[test]
fn existing_vapor_traversals_preserve_complete_diagnostics() {
    super::assert_traversal_parity(vapor_family, TEMPLATES, JSX);
}

#[test]
fn remaining_facade_traversals_preserve_complete_diagnostics() {
    super::assert_traversal_parity(
        remaining_facade_family,
        REMAINING_FACADE_TEMPLATES,
        REMAINING_FACADE_JSX,
    );
}

/// Recovered and implied-end-tag owners, including an ignored nested form and
/// a self-closing nested anchor, keep their v-pre subtree as Vue's compiler does.
/// Relief's tree construction dropped it in these shapes.
#[test]
fn recovered_v_pre_openers_follow_vue() {
    let rules = [
        "a11y/aria-unsupported-elements",
        "a11y/click-events-have-key-events",
        "a11y/no-static-element-interactions",
    ];
    let mut registry = crate::rule::RuleRegistry::new();
    registry.register(Box::new(AriaUnsupportedElements));
    registry.register(Box::new(ClickEventsHaveKeyEvents));
    registry.register(Box::new(NoStaticElementInteractions));
    let linter = crate::linter::Linter::with_registry(registry);
    let body = r#"<meta :role="x"><div @click="f"></div>"#;
    for (pre, control) in [
        (
            "<form><form v-pre>{}</form></form>",
            "<form><form>{}</form></form>",
        ),
        ("<a v-pre><a />{}</a>", "<a><a />{}</a>"),
        ("<p v-pre><div>{}</div></p>", "<p><div>{}</div></p>"),
        (
            "<select><option v-pre><option>{}</option></option></select>",
            "<select><option><option>{}</option></option></select>",
        ),
        (
            "<select><optgroup v-pre><optgroup>{}</optgroup></optgroup></select>",
            "<select><optgroup><optgroup>{}</optgroup></optgroup></select>",
        ),
        ("<li v-pre><li>{}</li></li>", "<li><li>{}</li></li>"),
        ("<dt v-pre><dd>{}</dd></dt>", "<dt><dd>{}</dd></dt>"),
    ] {
        for (shape, reported) in [(pre, Vec::new()), (control, rules.to_vec())] {
            let source = shape.replace("{}", body);
            let result = linter.lint_template(&source, "test.vue");
            let mut names: Vec<_> = result
                .diagnostics
                .iter()
                .map(|diagnostic| diagnostic.rule_name)
                .filter(|name| rules.contains(name))
                .collect();
            names.sort_unstable();
            assert_eq!(names, reported, "{source}: {:?}", result.diagnostics);
        }
    }
}
