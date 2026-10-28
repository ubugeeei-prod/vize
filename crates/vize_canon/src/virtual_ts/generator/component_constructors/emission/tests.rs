use super::{AliasReference, GenericContext, GenericSignature, SignatureRole};
use vize_carton::cstr;

#[test]
fn borrowed_aliases_preserve_complete_authored_arguments() {
    for (name, generic, arguments, expected) in [
        ("Emits", false, "T, U", "Emits"),
        ("$$Slots", true, "T, U", "$$Slots<T, U>"),
        ("Exposed", true, "", "Exposed<>"),
        ("Exposed", true, "項目, _Result", "Exposed<項目, _Result>"),
        (
            "Exposed",
            true,
            "T extends { key: 'a\\\"b' }",
            "Exposed<T extends { key: 'a\\\"b' }>",
        ),
        ("{}", false, "Ignored", "{}"),
    ] {
        assert_eq!(
            cstr!("{}", AliasReference::new(name, generic, arguments)),
            expected
        );
    }
}

#[test]
fn signature_roles_share_exact_context_grammar_and_keep_the_ref_probe_open() {
    let parameters = "<T extends string = 'a', 項目 = T>";
    let arguments = "<T, 項目>";
    for (role, expected) in [
        (
            SignatureRole::Component,
            "type $$GenericComponent = <T extends string = 'a', 項目 = T>(props: $$GenericContext<T, 項目>['props'], ctx?: Pick<$$GenericContext<T, 項目>, 'attrs' | 'emit' | 'slots'>, expose?: $$GenericContext<T, 項目>['expose']) => import('vue').VNode & { __ctx?: $$GenericContext<T, 項目> };\n",
        ),
        (
            SignatureRole::TemplateRefResolver,
            "type $$GenericRefResolver = <T extends string = 'a', 項目 = T>(props: $$GenericContext<T, 項目>['props'] & Record<string, unknown>, ctx?: Pick<$$GenericContext<T, 項目>, 'attrs' | 'emit' | 'slots'>, expose?: $$GenericContext<T, 項目>['expose']) => import('vue').VNode & { __ctx?: $$GenericContext<T, 項目> };\n",
        ),
    ] {
        assert_eq!(
            cstr!(
                "{}",
                GenericSignature {
                    role,
                    parameters,
                    arguments
                }
            ),
            expected
        );
    }
}

#[test]
fn signature_views_preserve_multiline_parameters_and_quoted_defaults() {
    for newline in ["\n", "\r\n"] {
        let parameters = "<T extends 'a\\\"b',\nU = T>".replace('\n', newline);
        let expected = "type $$GenericComponent = <T extends 'a\\\"b',\nU = T>(props: $$GenericContext<T, U>['props'], ctx?: Pick<$$GenericContext<T, U>, 'attrs' | 'emit' | 'slots'>, expose?: $$GenericContext<T, U>['expose']) => import('vue').VNode & { __ctx?: $$GenericContext<T, U> };\n";
        // Only the authored parameter list uses its original newline. The
        // generated declaration terminator remains the generator's LF.
        let expected = expected.replacen("b',\nU", &cstr!("b',{newline}U"), 1);
        assert_eq!(
            cstr!(
                "{}",
                GenericSignature {
                    role: SignatureRole::Component,
                    parameters: &parameters,
                    arguments: "<T, U>"
                }
            ),
            expected
        );
    }
}

#[test]
fn context_views_emit_complete_plain_and_refined_public_surfaces() {
    for (refined, expected) in [
        (
            false,
            "type $$GenericContext<T = any> = {\n  props: Props<T> & import('vue').VNodeProps & import('vue').AllowedComponentProps & import('vue').ComponentCustomProps;\n  attrs: any;\n  emit: {};\n  slots: $$Slots;\n  expose: (exposed: import('vue').ShallowUnwrapRef<{}>) => void;\n};\n",
        ),
        (
            true,
            "type $$GenericContext<T = any> = {\n  props: Props<T> & __EmitProps<Emits<T>> & $$JsxSlotProps<$$Slots<T>> & import('vue').VNodeProps & import('vue').AllowedComponentProps & import('vue').ComponentCustomProps;\n  attrs: any;\n  emit: $$PublicEmit<Emits<T>>;\n  slots: $$Slots<T>;\n  expose: (exposed: import('vue').ShallowUnwrapRef<Exposed<T>> & { $el: Awaited<ReturnType<typeof __setup<T>>>['__vize_root_el'] } & { $refs: Awaited<ReturnType<typeof __setup<T>>>['__vize_refs'] }) => void;\n};\n",
        ),
    ] {
        let context = GenericContext {
            parameters: "<T = any>",
            arguments: "<T>",
            props_type: "Props",
            emits: AliasReference::new("Emits", refined, "T"),
            slots: AliasReference::new("$$Slots", refined, "T"),
            exposed: AliasReference::new(if refined { "Exposed" } else { "{}" }, refined, "T"),
            emit_props: refined,
            jsx_slots: refined,
            root_element: refined,
            template_refs: refined,
        };
        assert_eq!(cstr!("{context}"), expected);
    }
}
