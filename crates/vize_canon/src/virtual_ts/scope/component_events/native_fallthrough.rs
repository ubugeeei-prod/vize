use vize_carton::{String, append};

pub(super) fn emit_args(
    ts: &mut String,
    indent: &str,
    fallthrough_args: &str,
    component_ref: &str,
    prop_listener_args: &str,
    is_any: &str,
) {
    append!(
        *ts,
        "{indent}type {fallthrough_args} = typeof {component_ref} extends {{ readonly __vizeNativeFallthroughRoot: true; readonly __vizeFallthroughProps?: infer __F }}\n",
    );
    append!(
        *ts,
        "{indent}  ? {is_any}<__F> extends true ? unknown[] : NonNullable<__F> extends infer __P ? ({prop_listener_args}) extends infer __A extends unknown[] ? (__A extends [] ? unknown[] : __A) : unknown[] : unknown[]\n",
    );
    append!(*ts, "{indent}  : unknown[];\n");
}

#[expect(
    clippy::too_many_arguments,
    reason = "the arguments are exact generated TypeScript identifiers"
)]
pub(super) fn emit_return(
    ts: &mut String,
    indent: &str,
    native_return: &str,
    return_type: &str,
    component_ref: &str,
    prop_args: &str,
    prop_key: &str,
    is_any: &str,
) {
    append!(
        *ts,
        "{indent}type {native_return} = typeof {component_ref} extends {{ readonly __vizeNativeFallthroughRoot: true; readonly __vizeFallthroughProps?: infer __F }} ? {is_any}<__F> extends true ? any : NonNullable<__F> extends {{ {prop_key}?: infer __H }} ? Extract<NonNullable<__H>, (...args: any[]) => any> extends (...args: any[]) => infer __R ? __R : any : any : any;\n",
    );
    append!(
        *ts,
        "{indent}type {return_type} = unknown[] extends {prop_args} ? {native_return} : typeof {component_ref} extends {{ new (...args: any[]): {{ $props: infer __P }} }}\n",
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
