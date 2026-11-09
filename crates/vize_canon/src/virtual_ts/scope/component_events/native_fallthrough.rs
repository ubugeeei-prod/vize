use vize_carton::append;

/// The native root's fallthrough props as a one-tuple, or `[]` when none apply.
pub(super) fn emit_box(
    ts: &mut crate::virtual_ts::TextBuf,
    indent: &str,
    fallthrough_box: &str,
    component_ref: &str,
    is_any: &str,
) {
    // A native root does not close an undeclared component event surface.
    // Test finite keys only after narrowing that root: generated Vue options
    // have a string index, and authored conditional exports need the narrowing.
    // An `any` root has no typed contract, the same as no root.
    append!(
        *ts,
        "{indent}type {fallthrough_box} = typeof {component_ref} extends {{ readonly __vizeNativeFallthroughRoot: true; readonly __vizeFallthroughProps?: infer __F }} ? \"__vizeEmitProps\" extends keyof {{ [__K in keyof typeof {component_ref} as string extends __K ? never : number extends __K ? never : symbol extends __K ? never : __K]: unknown }} ? {is_any}<__F> extends true ? [] : [__F] : [] : [];\n",
    );
}

pub(super) fn emit_args(
    ts: &mut crate::virtual_ts::TextBuf,
    indent: &str,
    fallthrough_args: &str,
    fallthrough_box: &str,
    prop_listener_args: &str,
) {
    append!(
        *ts,
        "{indent}type {fallthrough_args} = {fallthrough_box} extends infer __B ? __B extends [infer __F] ? NonNullable<__F> extends infer __P ? ({prop_listener_args}) extends infer __A extends unknown[] ? (__A extends [] ? unknown[] : __A) : unknown[] : unknown[] : unknown[] : never;\n",
    );
}

pub(super) fn emit_return(
    ts: &mut crate::virtual_ts::TextBuf,
    indent: &str,
    native_return: &str,
    fallthrough_box: &str,
    prop_key: &str,
) {
    append!(
        *ts,
        "{indent}type {native_return} = {fallthrough_box} extends infer __B ? __B extends [infer __F] ? NonNullable<__F> extends {{ {prop_key}?: infer __H }} ? Extract<NonNullable<__H>, (...args: any[]) => any> extends (...args: any[]) => infer __R ? __R : any : any : any : never;\n",
    );
}
