//! Compiles the real default native Canon surface from an independent root.

pub fn native_surface_layouts() -> [usize; 5] {
    [
        std::mem::size_of::<vize_canon::CorsaBridge>(),
        std::mem::size_of::<vize_canon::CorsaBridgeConfig>(),
        std::mem::size_of::<vize_canon::CorsaBridgeError>(),
        std::mem::size_of::<vize_canon::OriginalProgramCheck>(),
        std::mem::size_of::<vize_canon::corsa_client::CorsaProjectClient>(),
    ]
}

pub fn native_executor(
    root: &std::path::Path,
) -> Result<vize_canon::CorsaExecutor, vize_canon::CorsaNotFoundError> {
    vize_canon::CorsaExecutor::new(root)
}
