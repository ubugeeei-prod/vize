//! The SSR compile compiled a second time, for the render map the SFC
//! assembly composes into the component's (see [`crate::shared`]). Nothing in
//! the plain compile calls into this copy.

// This copy keeps the `composed` tokens of every `sfc_map_copy!`.
macro_rules! sfc_map_copy {
    (plain { $($plain:tt)* } composed { $($composed:tt)* }) => {
        $($composed)*
    };
}

#[path = "codegen/composed.rs"]
pub(crate) mod codegen;
#[path = "compile/composed.rs"]
pub mod compile;
#[path = "l4/composed.rs"]
pub(crate) mod l4;
