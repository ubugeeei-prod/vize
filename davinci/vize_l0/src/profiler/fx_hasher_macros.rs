//! Named private-map adoption; normal expansion is exactly the raw type.

/// Declare a private hasher alias and its profiling-only owner identity.
#[cfg(not(feature = "micro-profiling"))]
#[macro_export]
macro_rules! scoped_fx_hasher {
    ($alias:ident, $scope:ident, $prefix:literal) => {
        type $alias = $crate::FxBuildHasher;
    };
}

/// Declare a private hasher alias and source-owned stable interface counters.
#[cfg(feature = "micro-profiling")]
#[macro_export]
macro_rules! scoped_fx_hasher {
    ($alias:ident, $scope:ident, $prefix:literal) => {
        struct $scope;
        type $alias = $crate::profiler::fx_hasher::Builder<$scope>;
        impl $crate::profiler::fx_hasher::Scope for $scope {
            #[inline]
            fn record(operation: $crate::profiler::fx_hasher::Operation, units: u64) {
                match operation {
                    $crate::profiler::fx_hasher::Operation::Build => {
                        $crate::__scoped_fx_event!($prefix, ".build", units)
                    }
                    $crate::profiler::fx_hasher::Operation::HashOne => {
                        $crate::__scoped_fx_event!($prefix, ".hash_one", units)
                    }
                    $crate::profiler::fx_hasher::Operation::Write => {
                        $crate::__scoped_fx_event!($prefix, ".write.bytes", units)
                    }
                    $crate::profiler::fx_hasher::Operation::Finish => {
                        $crate::__scoped_fx_event!($prefix, ".finish", units)
                    }
                    $crate::profiler::fx_hasher::Operation::WriteU8 => {
                        $crate::__scoped_fx_event!($prefix, ".write_u8.bytes", units)
                    }
                    $crate::profiler::fx_hasher::Operation::WriteU16 => {
                        $crate::__scoped_fx_event!($prefix, ".write_u16.bytes", units)
                    }
                    $crate::profiler::fx_hasher::Operation::WriteU32 => {
                        $crate::__scoped_fx_event!($prefix, ".write_u32.bytes", units)
                    }
                    $crate::profiler::fx_hasher::Operation::WriteU64 => {
                        $crate::__scoped_fx_event!($prefix, ".write_u64.bytes", units)
                    }
                    $crate::profiler::fx_hasher::Operation::WriteU128 => {
                        $crate::__scoped_fx_event!($prefix, ".write_u128.bytes", units)
                    }
                    $crate::profiler::fx_hasher::Operation::WriteUsize => {
                        $crate::__scoped_fx_event!($prefix, ".write_usize.bytes", units)
                    }
                    $crate::profiler::fx_hasher::Operation::WriteI8 => {
                        $crate::__scoped_fx_event!($prefix, ".write_i8.bytes", units)
                    }
                    $crate::profiler::fx_hasher::Operation::WriteI16 => {
                        $crate::__scoped_fx_event!($prefix, ".write_i16.bytes", units)
                    }
                    $crate::profiler::fx_hasher::Operation::WriteI32 => {
                        $crate::__scoped_fx_event!($prefix, ".write_i32.bytes", units)
                    }
                    $crate::profiler::fx_hasher::Operation::WriteI64 => {
                        $crate::__scoped_fx_event!($prefix, ".write_i64.bytes", units)
                    }
                    $crate::profiler::fx_hasher::Operation::WriteI128 => {
                        $crate::__scoped_fx_event!($prefix, ".write_i128.bytes", units)
                    }
                    $crate::profiler::fx_hasher::Operation::WriteIsize => {
                        $crate::__scoped_fx_event!($prefix, ".write_isize.bytes", units)
                    }
                }
            }
        }
    };
}

/// Implementation detail: each expansion registers one exact owner source site.
#[cfg(feature = "micro-profiling")]
#[doc(hidden)]
#[macro_export]
macro_rules! __scoped_fx_event {
    ($prefix:literal, $suffix:literal, $units:expr) => {{
        static SITE: $crate::profiler::micro::Site = $crate::profiler::micro::Site::new(
            concat!($prefix, $suffix, "@", file!(), ":", line!()),
            concat!($prefix, $suffix),
            file!(),
            module_path!(),
            line!(),
            "counter",
        );
        $crate::profiler::micro::count(&SITE, $units);
    }};
}
