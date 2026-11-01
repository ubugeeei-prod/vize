use super::*;

const CONSTRUCTOR: &str = "export type Props = {title: string};\ntype $ComponentConstructor = new <$AuthoredProps = unknown>(props?: $AuthoredProps & $ComponentInput<Props, {}, $AuthoredProps>, ...args: any[]) => $UsePublicInstance<$AuthoredProps> extends true ? $ComponentInstance : $ReplaceComponentProps<$ComponentInstance, $AuthoredKeyWitness<$AuthoredProps> & $ComponentInputProps<Props, {}> & $ComponentInputGuard<Props, {}, $AuthoredProps>>;\n";

#[test]
fn captures_the_whole_original_independent_return_without_a_new_binding() {
    let source = vize_carton::cstr!("export {{}};\n{CONSTRUCTOR}");
    let site = analyze(&source).unwrap();
    assert_eq!(site.props, "Props");
    assert_eq!(site.declared, "{}");
    assert_eq!(
        source.get(site.range).unwrap(),
        "$UsePublicInstance<$AuthoredProps> extends true ? $ComponentInstance : $ReplaceComponentProps<$ComponentInstance, $AuthoredKeyWitness<$AuthoredProps> & $ComponentInputProps<Props, {}> & $ComponentInputGuard<Props, {}, $AuthoredProps>>"
    );
}

#[test]
fn all_moved_names_and_escaped_or_imported_bindings_refuse() {
    for name in MOVED_NAMES {
        for declaration in [
            vize_carton::cstr!("type {name}<T> = T;"),
            vize_carton::cstr!("import type {{ H as {name} }} from './types';"),
            vize_carton::cstr!("function outer<{name}>() {{}}"),
            vize_carton::cstr!("void (null as {name}<unknown>);"),
        ] {
            assert!(analyze(&vize_carton::cstr!("{declaration}\n{CONSTRUCTOR}")).is_none());
        }
    }
    assert!(
        analyze(&vize_carton::cstr!(
            "type \\u0024UsePublicInstance<T> = T;\n{CONSTRUCTOR}"
        ))
        .is_none()
    );
}

#[test]
fn component_input_alone_does_not_change_return_helper_eligibility() {
    assert!(
        analyze(&vize_carton::cstr!(
            "type $ComponentInput<T,D,A> = {{ authored: true }};\n{CONSTRUCTOR}"
        ))
        .is_some()
    );
}

#[test]
fn changed_distribution_defaults_duplicates_and_opaque_sources_refuse() {
    for source in [
        String::from(CONSTRUCTOR.replace(" & $ComponentInputGuard", " | $ComponentInputGuard")),
        String::from(CONSTRUCTOR.replace("$AuthoredProps = unknown", "$AuthoredProps = any")),
        vize_carton::cstr!("{CONSTRUCTOR}{CONSTRUCTOR}"),
        vize_carton::cstr!("declare module 'other' {{}}\n{CONSTRUCTOR}"),
        vize_carton::cstr!("void import(dynamicName);\n{CONSTRUCTOR}"),
        vize_carton::cstr!("type Broken = ;\n{CONSTRUCTOR}"),
    ] {
        assert!(analyze(&source).is_none(), "{source}");
    }
}

#[test]
fn raw_generic_arity_errors_are_never_deduplicated_by_factoring() {
    for source in [
        CONSTRUCTOR.replace("export type Props =", "export type Props<T> ="),
        CONSTRUCTOR.replace("Props, {}", "NeedsArgument, {}"),
        CONSTRUCTOR.replace("Props, {}", "Props, NeedsArgument"),
        vize_carton::cstr!("{CONSTRUCTOR}\ntype Props = {{poison: true}};"),
        String::from(CONSTRUCTOR.replace("export type Props = {title: string};\n", "")),
    ] {
        assert!(analyze(&source).is_none(), "{source}");
    }
}
