use std::path::Path;

pub(super) fn assert_generation_order(
    path: &Path,
    trace: &[u8],
    did_open: usize,
    shutdown: usize,
    exit: usize,
    expected_generations: usize,
) -> Result<(), String> {
    let ready = find_all_bytes(trace, b"textDocument/diagnostic");
    if ready.len() != expected_generations {
        return Err(format!(
            "expected {expected_generations} readiness requests, found {} in {}",
            ready.len(),
            path.display()
        ));
    }
    let first_ready = ready.first().copied().ok_or_else(|| {
        format!(
            "missing exact document-readiness request before rename in {}",
            path.display()
        )
    })?;
    let renames = find_all_bytes(trace, b"textDocument/rename");
    let first_rename = renames
        .first()
        .copied()
        .ok_or_else(|| format!("missing rename request in {}", path.display()))?;
    if !(did_open < first_ready
        && first_ready < first_rename
        && renames.iter().all(|rename| *rename < shutdown)
        && shutdown < exit)
    {
        return Err(format!(
            "invalid editor LSP lifecycle order in {}: didOpen={did_open}, ready={ready:?}, rename={renames:?}, shutdown={shutdown}, exit={exit}",
            path.display()
        ));
    }
    if expected_generations == 2 {
        assert_cross_document_generation(path, trace, &ready, &renames, shutdown)?;
    }
    Ok(())
}

fn assert_cross_document_generation(
    path: &Path,
    trace: &[u8],
    ready: &[usize],
    renames: &[usize],
    shutdown: usize,
) -> Result<(), String> {
    let did_change = super::find_bytes(trace, b"textDocument/didChange")
        .ok_or_else(|| format!("missing cross-document didChange in {}", path.display()))?;
    let first_generation = renames
        .iter()
        .filter(|rename| ready[0] < **rename && **rename < did_change)
        .count();
    let second_generation = renames
        .iter()
        .filter(|rename| ready[1] < **rename && **rename < shutdown)
        .count();
    if ready[0] < did_change
        && did_change < ready[1]
        && first_generation >= 2
        && second_generation >= 2
    {
        return Ok(());
    }
    Err(format!(
        "invalid cross-document readiness order in {}: ready={ready:?}, didChange={did_change}, rename={renames:?}, shutdown={shutdown}",
        path.display()
    ))
}

fn find_all_bytes(haystack: &[u8], needle: &[u8]) -> Vec<usize> {
    haystack
        .windows(needle.len())
        .enumerate()
        .filter_map(|(index, window)| (window == needle).then_some(index))
        .collect()
}
