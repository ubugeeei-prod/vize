use super::*;

fn is_camel_or_pascal(spelling: &str) -> bool {
    spelling
        .strip_prefix('$')
        .is_some_and(|rest| !rest.is_empty() && rest.bytes().all(|b| b.is_ascii_alphanumeric()))
}

#[test]
fn every_name_is_one_sigil_and_its_real_name() {
    for name in ALL {
        assert!(is_camel_or_pascal(name.as_str()), "{name:?}");
        let mentions_vize = name.as_str().to_ascii_lowercase().find("vize");
        assert_eq!(mentions_vize, None, "{name:?}");
    }
}

#[test]
fn names_never_equal_vue_instance_members() {
    for name in ALL {
        let reserved = RESERVED.iter().position(|member| *member == name.as_str());
        assert_eq!(reserved, None, "{name:?}");
    }
}

#[test]
fn names_and_families_never_overlap() {
    for (index, first) in ALL.iter().enumerate() {
        for second in ALL.iter().skip(index + 1) {
            assert_ne!(first.as_str(), second.as_str());
            for (stem, other) in [(first, second), (second, first)] {
                assert!(
                    !stem.is_member(other.as_str()),
                    "{other:?} reads as a member of {stem:?}"
                );
            }
        }
    }
}

#[test]
fn recognition_accepts_members_and_hygiene_spellings() {
    assert!(TEMPLATE_INSTANCE.is("$inst"));
    assert!(TEMPLATE_INSTANCE.is("$inst$$"));
    assert!(!TEMPLATE_INSTANCE.is("$instance"));
    assert!(PROP_CHECK.is_member("$prop0_label"));
    assert!(PROP_CHECK.is_member("$prop$0_label"));
    assert!(!PROP_CHECK.is_member("$props"));
    assert!(!PROP_CHECK.is_member("$propShape"));
    assert_eq!(PROPS_NAV.strip("$propsNav3;"), Some("3;"));
    assert!(is_generated("$propsNav3"));
    assert!(is_generated("$args"));
    assert!(!is_generated("$attrs"));
    assert!(!is_generated("$props"));
    assert!(!is_generated("args"));
}

#[test]
fn literal_macro_matches_the_constant() {
    assert_eq!(
        generated_name!(TEMPLATE_INSTANCE),
        TEMPLATE_INSTANCE.as_str()
    );
    assert_eq!(
        concat!("void ", generated_name!(PROPS_NAV), "0;"),
        "void $propsNav0;"
    );
}

#[test]
fn unbound_names_keep_their_spelling() {
    assert_eq!(crate::cstr!("{LISTENER_ARGS}"), "$args");
    assert_eq!(crate::cstr!("{PROPS_NAV}{}", 2), "$propsNav2");
    assert_eq!(
        hygiene::text("(...$args) => $args[0]"),
        "(...$args) => $args[0]"
    );
}

#[test]
fn authored_identifiers_take_the_generated_spelling() {
    let source = "const $args = 1; const $args$ = 2; $attrs.class; a$b; $propsNav0";
    let authored: Vec<&str> = hygiene::dollar_identifiers(source).collect();
    assert_eq!(authored, ["$args", "$args$", "$attrs", "$propsNav0"]);
    {
        let _scope = hygiene::bind(authored.iter().copied());
        assert_eq!(crate::cstr!("{LISTENER_ARGS}"), "$args$$");
        assert_eq!(crate::cstr!("{PROPS_NAV}{}", 0), "$propsNav$0");
        assert_eq!(crate::cstr!("{CALLBACK}"), "$cb");
        assert_eq!(
            hygiene::text("(...$args) => $args[0] + $cb"),
            "(...$args$$) => $args$$[0] + $cb"
        );
        assert!(LISTENER_ARGS.is("$args$$"));
        let mut emitted = crate::String::default();
        crate::ts_emit::write_hole(&mut emitted, &LISTENER_ARGS);
        assert_eq!(emitted.len(), crate::ts_emit::hole_len(&LISTENER_ARGS));
        assert_eq!(emitted, "$args$$");
        {
            let _inner = hygiene::bind([]);
            assert_eq!(crate::cstr!("{LISTENER_ARGS}"), "$args");
        }
        assert_eq!(crate::cstr!("{LISTENER_ARGS}"), "$args$$");
    }
    assert_eq!(crate::cstr!("{LISTENER_ARGS}"), "$args");
    assert!(!hygiene::is_active());
}
