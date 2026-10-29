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
    pub(super) component_ref: &'a str,
    pub(super) prop_key: &'a str,
    pub(super) is_any: &'a str,
}

pub(super) fn emit(ts: &mut String, ctx: Context<'_>) -> String {
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
        component_ref,
        prop_key,
        is_any,
    } = ctx;
    if legacy_vue2 {
        return cstr!("(...args: {listener_args_type}) => unknown");
    }
    if first_contract {
        if native_fallthrough_only {
            let native_return = cstr!("{return_type}_native_fallthrough");
            super::native_fallthrough::emit_return(
                ts,
                indent,
                native_return.as_str(),
                return_type,
                component_ref,
                prop_args,
                prop_key,
                is_any,
            );
        } else if let Some(props) = contextual_props {
            append!(
                *ts,
                "{indent}type {return_type} = unknown[] extends {prop_args} ? any : {props} extends {{ {prop_key}?: infer __H }} ? Extract<NonNullable<__H>, (...args: any[]) => any> extends (...args: any[]) => infer __R ? __R : any : any;\n",
            );
        } else {
            append!(
                *ts,
                "{indent}type {return_type} = unknown[] extends {prop_args} ? any : typeof {component_ref} extends {{ new (...args: any[]): {{ $props: infer __P }} }}\n",
            );
            append!(
                *ts,
                "{indent}  ? (__P extends {{ {prop_key}?: (...args: any[]) => infer __R }} ? __R : any)\n",
            );
            append!(
                *ts,
                "{indent}  : typeof {component_ref} extends (props: infer __P, ...args: any[]) => any\n",
            );
            append!(
                *ts,
                "{indent}    ? (__P extends {{ {prop_key}?: (...args: any[]) => infer __R }} ? __R : any)\n",
            );
            append!(*ts, "{indent}    : any;\n");
        }
    }
    cstr!(
        "unknown[] extends {args_type} ? ((...args: any[]) => any) : ((...args: {listener_args_type}) => {return_type})"
    )
}
