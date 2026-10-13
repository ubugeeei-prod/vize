//! Compiles Canon's real default native surface from an independent Cargo root.

pub fn native_surface_layouts() -> [usize; 5] {
    [
        std::mem::size_of::<vize_canon::CorsaBridge>(),
        std::mem::size_of::<vize_canon::CorsaBridgeConfig>(),
        std::mem::size_of::<vize_canon::CorsaBridgeError>(),
        std::mem::size_of::<vize_canon::OriginalProgramCheck>(),
        std::mem::size_of::<vize_canon::corsa_client::CorsaProjectClient>(),
    ]
}
