//! The disabled definitions do not evaluate counter operands or read a clock.

/// Count one source callsite and sum its explicitly supplied unit (e.g. bytes).
#[cfg(feature = "micro-profiling")]
#[macro_export]
macro_rules! micro_count {
    ($name:literal, $value:expr) => {{
        static SITE: $crate::profiler::micro::Site = $crate::profiler::micro::Site::new(
            concat!($name, "@", file!(), ":", line!()),
            $name,
            file!(),
            module_path!(),
            line!(),
            "counter",
        );
        $crate::profiler::micro::count(&SITE, $value as u64);
    }};
}

/// Executes its block unchanged outside the `micro-profiling` build feature.
#[cfg(feature = "micro-profiling")]
#[macro_export]
macro_rules! micro_span {
    ($name:literal, $block:expr) => {{
        static SITE: $crate::profiler::micro::Site = $crate::profiler::micro::Site::new(
            concat!($name, "@", file!(), ":", line!()),
            $name,
            file!(),
            module_path!(),
            line!(),
            "span",
        );
        let _micro_guard = $crate::profiler::micro::span(&SITE);
        $block
    }};
}

#[cfg(not(feature = "micro-profiling"))]
#[macro_export]
macro_rules! micro_count {
    ($name:literal, $value:expr) => {{}};
}

#[cfg(not(feature = "micro-profiling"))]
#[macro_export]
macro_rules! micro_span {
    ($name:literal, $block:expr) => {{ $block }};
}
