use super::matrix::Fixture;

pub(super) fn retain(fixture: &Fixture, lane: &str, parts: &[(&str, &str)]) {
    if let Err(error) = write(fixture, lane, parts) {
        panic!("failed to retain complete TS-40 projection: {error}");
    }
}

fn write(fixture: &Fixture, lane: &str, parts: &[(&str, &str)]) -> std::io::Result<()> {
    let Some(root) = std::env::var_os("VIZE_TEMPLATE_EMIT_TS40_CAPTURE") else {
        return Ok(());
    };
    let root = std::path::PathBuf::from(root).join(fixture.id.as_str());
    std::fs::create_dir_all(&root)?;
    std::fs::write(root.join("original.vue"), fixture.source().as_bytes())?;
    let root = root.join(lane);
    std::fs::create_dir_all(&root)?;
    for (name, bytes) in parts {
        std::fs::write(root.join(name), bytes)?;
    }
    Ok(())
}
