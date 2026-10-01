//! Resolved native policy facts belong to the existing opening owner.

use vize_l0::{Allocator, cstr};
use vize_l1::markup::{parse_component, parse_component_with_authored};
use vize_l1::{Element, OpenTag, SurfaceChild, Token, check_fidelity};

fn modes<'a>(children: &[SurfaceChild<'a>]) -> Vec<(&'a str, bool)> {
    let mut result = Vec::new();
    fn collect<'a>(children: &[SurfaceChild<'a>], result: &mut Vec<(&'a str, bool)>) {
        for child in children {
            if let SurfaceChild::Element(element) = child {
                result.push((element.tag(), element.open.is_verbatim()));
                collect(&element.children, result);
            }
        }
    }
    collect(children, &mut result);
    result
}

#[test]
fn native_opening_facts_include_own_inherited_void_and_self_closing_modes() {
    for head in [
        "v-pre",
        "v-pre.foo",
        "v-pre:arg",
        "v-pre:[key]",
        "v-pre:[broken",
        "v-pre:",
    ] {
        let source = cstr!(
            "<section><div {head} single='日' double=\"中\" bare=value><span/>{{{{raw}}}}<input/></div><p>{{{{normal}}}}</p></section>"
        );
        let allocator = Allocator::new();
        let parsed = parse_component(&allocator, &source).unwrap();
        assert_eq!(check_fidelity(&parsed.tree), Ok(()));
        assert!(parsed.unsupported.is_empty());
        assert_eq!(
            modes(&parsed.tree.children),
            [
                ("section", false),
                ("div", true),
                ("span", true),
                ("input", true),
                ("p", false)
            ],
            "{source}"
        );
        let SurfaceChild::Element(section) = &parsed.tree.children[0] else {
            panic!("section")
        };
        let SurfaceChild::Element(div) = &section.children[0] else {
            panic!("div")
        };
        for (attribute, quote) in div.open.attrs.iter().skip(1).zip(["'", "\"", ""]) {
            let value = attribute.value.as_ref().unwrap();
            assert_eq!(
                value.open_quote.as_ref().map_or("", |token| token.text),
                quote
            );
            assert_eq!(
                value.close_quote.as_ref().map_or("", |token| token.text),
                quote
            );
        }
    }
}

#[test]
fn recovery_fact_follows_live_owner_for_both_projections() {
    let source = "<a v-pre><span>{{before}}<a>{{after}}</a>{{tail}}</span></a><p>{{end}}</p>";
    let allocator = Allocator::new();
    let parsed = parse_component_with_authored(&allocator, source).unwrap();
    assert_eq!(
        modes(&parsed.tree.children),
        [("a", true), ("span", true), ("a", false), ("p", false)]
    );
    assert_eq!(check_fidelity(&parsed.tree), Ok(()));
    let authored = parsed.authored.as_ref().unwrap();
    assert_eq!(
        modes(&authored.children),
        [("a", true), ("span", true), ("a", false), ("p", false)]
    );
    assert_eq!(check_fidelity(authored), Ok(()));
}

#[test]
fn raw_construction_compatibility_and_public_debug_do_not_forge_metadata() {
    let raw_allocator = Allocator::new();
    let raw = OpenTag {
        lt_name: Token::present("", "<div"),
        attrs: vize_l0::Vec::new_in(&&raw_allocator),
        slash: None,
        gt: Token::present("", ">"),
    };
    assert!(!raw.is_verbatim());
    let allocator = Allocator::new();
    let source = "<div v-pre>{{raw}}</div>";
    let native = parse_component(&allocator, source).unwrap();
    let compat = vize_l1::parse(&allocator, source);
    assert_eq!(modes(&compat.0.children), [("div", false)]);
    let SurfaceChild::Element(div) = &native.tree.children[0] else {
        panic!("div")
    };
    assert!(div.open.is_verbatim());
    assert_eq!(
        cstr!("{:?}", div.open.lt_name),
        "Token { leading: \"\", text: \"<div\", status: Present }"
    );
    let SurfaceChild::Element(compat_div) = &compat.0.children[0] else {
        panic!("compat div")
    };
    assert_eq!(cstr!("{:?}", div.open), cstr!("{:?}", compat_div.open));
}

#[test]
fn private_fact_fits_existing_surface_layout_caps() {
    if cfg!(target_pointer_width = "64") {
        assert_eq!(core::mem::size_of::<Token<'_>>(), 40);
        assert_eq!(core::mem::size_of::<Option<Token<'_>>>(), 40);
        assert_eq!(core::mem::size_of::<OpenTag<'_>>(), 144);
        assert_eq!(core::mem::size_of::<Element<'_>>(), 248);
    }
}

#[test]
fn resolved_mode_never_invents_admission_for_an_overbudget_sole_control() {
    let argument = cstr!("[{}{}]", "([".repeat(33), "])".repeat(33));
    let source = cstr!("<div v-pre:{argument}>{{{{normal}}}}</div>");
    let allocator = Allocator::new();
    let parsed = parse_component(&allocator, &source).unwrap();
    assert_eq!(parsed.unsupported.len(), 1);
    assert_eq!(
        parsed.unsupported[0].error,
        vize_l1::markup::DirectiveNameError::NestingLimit
    );
    assert_eq!(modes(&parsed.tree.children), [("div", false)]);
    assert_eq!(check_fidelity(&parsed.tree), Ok(()));
    for attributes in [
        cstr!("v-bind:{argument} v-pre"),
        cstr!("v-pre v-bind:{argument}"),
    ] {
        let source = cstr!("<div {attributes}><span>{{{{raw}}}}</span></div><p>{{{{normal}}}}</p>");
        let allocator = Allocator::new();
        let parsed = parse_component(&allocator, &source).unwrap();
        assert!(parsed.unsupported.is_empty());
        assert_eq!(
            modes(&parsed.tree.children),
            [("div", true), ("span", true), ("p", false)]
        );
        assert_eq!(check_fidelity(&parsed.tree), Ok(()));
    }
}

#[test]
fn no_control_proof_and_utf8_recovery_cuts_never_invent_verbatim_mode() {
    for head in [
        "v-pre[broken",
        "v-pre[key].foo",
        "v-pretty",
        "v-PRE",
        "@pre",
        ":pre",
        "#pre",
        ".pre",
    ] {
        let source = cstr!("<div {head}>{{{{normal}}}}</div>");
        let allocator = Allocator::new();
        let parsed = parse_component(&allocator, &source).unwrap();
        assert_eq!(modes(&parsed.tree.children), [("div", false)]);
        assert_eq!(check_fidelity(&parsed.tree), Ok(()));
    }
    let source = "中🍣<div v-pre title='未完';><span/>{{raw}}</div><p>{{normal}}</p>";
    for end in source
        .char_indices()
        .map(|(index, _)| index)
        .chain([source.len()])
    {
        let allocator = Allocator::new();
        let parsed = parse_component(&allocator, &source[..end]).unwrap();
        assert_eq!(check_fidelity(&parsed.tree), Ok(()), "{end}");
        let actual = match parsed.tree.children.get(1) {
            Some(SurfaceChild::Element(owner)) => Some((owner.tag(), owner.open.is_verbatim())),
            _ => None,
        };
        // The authored control name ends at byte 17; earlier cuts retain partial names.
        let expected = match end {
            0..=8 => None,
            9 => Some(("d", false)),
            10 => Some(("di", false)),
            11..=16 => Some(("div", false)),
            _ => Some(("div", true)),
        };
        assert_eq!(actual, expected, "{end}");
    }
}
