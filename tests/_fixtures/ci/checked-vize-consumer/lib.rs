//! Compile a CLI library entrypoint and its default native Canon re-exports.

pub fn cli_entrypoint() -> fn(Vec<String>) {
    vize::cli::run_from_args
}

pub fn native_canon_layouts() -> [usize; 2] {
    [
        std::mem::size_of::<vize::canon::CorsaBridge>(),
        std::mem::size_of::<vize::canon::corsa_client::CorsaProjectClient>(),
    ]
}
