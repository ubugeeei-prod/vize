use std::collections::BTreeSet;

use tower_lsp::lsp_types::Url;

pub(super) fn assert_mirror_root(
    trace: &[u8],
    canonical_source_root: &str,
    logical_source_root: &str,
) -> Result<(), String> {
    let messages = super::super::evidence::client_messages(trace)?;
    let initialize = messages
        .iter()
        .filter(|message| message["method"] == "initialize")
        .collect::<Vec<_>>();
    if initialize.len() != 1 {
        return Err(format!(
            "expected one initialization, found {}",
            initialize.len()
        ));
    }
    let root_uri = initialize[0]["params"]["rootUri"]
        .as_str()
        .ok_or("missing root URI")?;
    let root = Url::parse(root_uri)
        .map_err(|error| error.to_string())?
        .to_file_path()
        .map_err(|()| "mirror root is not a file URI")?;
    let private_sessions = std::env::temp_dir()
        .canonicalize()
        .map_err(|error| error.to_string())?
        .join("vize-canon/editor/sessions");
    let relative = root.strip_prefix(&private_sessions).map_err(|_| {
        format!("editor root is outside the physical session namespace: {root_uri}")
    })?;
    if !relative
        .components()
        .next()
        .is_some_and(|part| part.as_os_str().to_string_lossy().starts_with("session-"))
    {
        return Err(format!(
            "editor root has no private session owner: {root_uri}"
        ));
    }
    let mut opened = BTreeSet::new();
    for message in messages
        .iter()
        .filter(|message| message["method"] == "textDocument/didOpen")
    {
        let uri = message["params"]["textDocument"]["uri"]
            .as_str()
            .ok_or("missing didOpen URI")?;
        if uri.starts_with(canonical_source_root) || uri.starts_with(logical_source_root) {
            return Err(format!(
                "editor mixed authored and materialized identities: {uri}"
            ));
        }
        let path = Url::parse(uri)
            .map_err(|error| error.to_string())?
            .to_file_path()
            .map_err(|()| "document is not a file URI")?;
        let relative = path
            .strip_prefix(&root)
            .map_err(|_| format!("opened document escaped this native project: {uri}"))?;
        opened.insert(relative.to_string_lossy().replace('\\', "/"));
    }
    let expected = BTreeSet::from([
        "src/Parent.vue.ts".to_owned(),
        "src/Child.vue.ts".to_owned(),
    ]);
    if opened != expected {
        return Err(format!("unexpected editor project members: {opened:?}"));
    }
    Ok(())
}
