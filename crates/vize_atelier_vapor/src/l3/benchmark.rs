//! Detailed attribution is absent from ordinary builds and timed benchmark
//! builds. Only the separate `level-benchmark-profile` binary records it.

#[cfg(feature = "level-benchmark-profile")]
macro_rules! bridge_profile {
    ($name:literal, $body:expr) => {
        vize_carton::profile!($name, $body)
    };
}

#[cfg(not(feature = "level-benchmark-profile"))]
macro_rules! bridge_profile {
    ($name:literal, $body:expr) => {
        $body
    };
}

pub(super) use bridge_profile;
