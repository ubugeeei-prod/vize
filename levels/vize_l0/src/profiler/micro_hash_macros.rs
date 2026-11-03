//! Selected std hasher instrumentation is entirely absent from normal builds.

/// Count builds, byte-write operations/bytes and finish calls separately.
/// Only replace known `std::hash::DefaultHasher::new()` callsites with this.
#[cfg(feature = "micro-profiling")]
#[macro_export]
macro_rules! micro_default_hasher {
    ($name:literal) => {{
        static BUILDS: $crate::profiler::micro::Site = $crate::profiler::micro::Site::new(
            concat!($name, ".build@", file!(), ":", line!()),
            concat!($name, ".build"),
            file!(),
            module_path!(),
            line!(),
            "counter",
        );
        static WRITES: $crate::profiler::micro::Site = $crate::profiler::micro::Site::new(
            concat!($name, ".write_bytes@", file!(), ":", line!()),
            concat!($name, ".write_bytes"),
            file!(),
            module_path!(),
            line!(),
            "counter",
        );
        static FINISHES: $crate::profiler::micro::Site = $crate::profiler::micro::Site::new(
            concat!($name, ".finish@", file!(), ":", line!()),
            concat!($name, ".finish"),
            file!(),
            module_path!(),
            line!(),
            "counter",
        );
        $crate::profiler::micro_hash::DefaultHasherProbe::new(&BUILDS, &WRITES, &FINISHES)
    }};
}

#[cfg(not(feature = "micro-profiling"))]
#[macro_export]
macro_rules! micro_default_hasher {
    ($name:literal) => {{ std::hash::DefaultHasher::new() }};
}
