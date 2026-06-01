/// Returns the Vue runtime boundary encoded by the SFC filename.
///
/// Boundary filenames are resolved before SFC compilation, so keeping this
/// logic native gives every JS hook the same suffix interpretation.
pub fn boundary_kind(path: &str) -> Option<&'static str> {
    if path.ends_with(".client.vue") {
        Some("client")
    } else if path.ends_with(".server.vue") {
        Some("server")
    } else {
        None
    }
}
