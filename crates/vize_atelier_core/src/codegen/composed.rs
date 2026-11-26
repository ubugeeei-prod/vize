// The DOM generator compiled a second time, for the render map the SFC
// assembly composes into the component's (see `crate::composed_codegen`).
// This copy keeps the `composed` tokens of every `render_map_copy!`.
macro_rules! render_map_copy {
    (plain { $($plain:tt)* } composed { $($composed:tt)* }) => {
        $($composed)*
    };
}
include!("mod.rs");
