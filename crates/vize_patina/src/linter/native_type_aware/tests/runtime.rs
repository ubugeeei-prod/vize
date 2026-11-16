//! Native TypeScript runtime availability shared by type-aware lint tests.

pub(in crate::linter::native_type_aware) fn corsa_available() -> bool {
    let mut session =
        match super::super::CorsaTypeAwareSession::new_with_corsa_path("Component.vue", None, None)
        {
            Ok(session) => session,
            Err(_) => return false,
        };
    if session
        .open_virtual_project("const value = 1;\n", "Component.vue")
        .is_err()
    {
        session.close();
        return false;
    }
    session.close();
    true
}
