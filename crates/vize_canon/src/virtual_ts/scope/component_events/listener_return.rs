use vize_carton::{String, append, cstr};

pub(super) struct Context<'a> {
    pub(super) legacy_vue2: bool,
    pub(super) first_contract: bool,
    pub(super) native_fallthrough_only: bool,
    pub(super) contextual_props: Option<&'a str>,
    pub(super) indent: &'a str,
    pub(super) listener_args_type: &'a str,
    pub(super) args_type: &'a str,
    pub(super) return_type: &'a str,
    pub(super) prop_args: &'a str,
    pub(super) props_box: &'a str,
    pub(super) fallthrough_box: &'a str,
    pub(super) prop_key: &'a str,
}

pub(super) fn emit(ts: &mut crate::virtual_ts::TextBuf, ctx: Context<'_>) -> String {
    let Context {
        legacy_vue2,
        first_contract,
        native_fallthrough_only,
        contextual_props,
        indent,
        listener_args_type,
        args_type,
        return_type,
        prop_args,
        props_box,
        fallthrough_box,
        prop_key,
    } = ctx;
    if legacy_vue2 {
        return cstr!("(...args: {listener_args_type}) => unknown");
    }
    if first_contract {
        let unresolved_return = if native_fallthrough_only {
            let native_return = cstr!("{return_type}_native_fallthrough");
            super::native_fallthrough::emit_return(
                ts,
                indent,
                native_return.as_str(),
                fallthrough_box,
                prop_key,
            );
            native_return
        } else {
            String::from("any")
        };
        if let Some(props) = contextual_props {
            let lib_non_nullable = crate::virtual_ts::generated_names::lib("NonNullable");
            let lib_extract = crate::virtual_ts::generated_names::lib("Extract");
            append!(
                *ts,
                "{indent}type {return_type} = unknown[] extends {prop_args} ? any : {props} extends {{ {prop_key}?: infer __H }} ? {lib_extract}<{lib_non_nullable}<__H>, (...args: any[]) => any> extends (...args: any[]) => infer __R ? __R : any : any;\n",
            );
        } else {
            append!(
                *ts,
                "{indent}type {return_type} = unknown[] extends {prop_args} ? {unresolved_return} : {props_box} extends infer __B ? __B extends [infer __P] ? (__P extends {{ {prop_key}?: (...args: any[]) => infer __R }} ? __R : any) : any : never;\n",
            );
        }
    }
    cstr!(
        "unknown[] extends {args_type} ? ((...args: any[]) => any) : ((...args: {listener_args_type}) => {return_type})"
    )
}
