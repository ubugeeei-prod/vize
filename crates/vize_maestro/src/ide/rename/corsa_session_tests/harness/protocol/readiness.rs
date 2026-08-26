use std::path::Path;

use vize_s0::cstr;

pub(super) fn assert_generation_order(
    path: &Path,
    trace: &[u8],
    did_open: usize,
    shutdown: usize,
    exit: usize,
    expected_generations: usize,
) -> Result<(), String> {
    let ready = find_all_bytes(trace, b"textDocument/diagnostic");
    // The initial generation opens both Parent and Child. Every later
    // generation in this fixture changes only Child, so each adds one exact
    // readiness request.
    let expected_requests = expected_generations + 1;
    if ready.len() != expected_requests {
        return Err(cstr!(
            "expected {expected_requests} readiness requests for {expected_generations} generations, found {} in {}",
            ready.len(),
            path.display()
        ).into());
    }
    let first_ready = ready.first().copied().ok_or_else(|| {
        cstr!(
            "missing exact document-readiness request before rename in {}",
            path.display()
        )
    })?;
    let renames = find_all_bytes(trace, b"textDocument/rename");
    let first_rename = renames
        .first()
        .copied()
        .ok_or_else(|| cstr!("missing rename request in {}", path.display()))?;
    let first_generation_ready = ready[1];
    if !(did_open < first_ready
        && first_ready < first_generation_ready
        && first_generation_ready < first_rename
        && renames.iter().all(|rename| *rename < shutdown)
        && shutdown < exit)
    {
        return Err(cstr!(
            "invalid editor LSP lifecycle order in {}: didOpen={did_open}, ready={ready:?}, rename={renames:?}, shutdown={shutdown}, exit={exit}",
            path.display()
        ).into());
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
        .ok_or_else(|| cstr!("missing cross-document didChange in {}", path.display()))?;
    let first_generation_ready = ready[1];
    let second_generation_ready = ready[2];
    let first_generation = renames
        .iter()
        .filter(|rename| first_generation_ready < **rename && **rename < did_change)
        .count();
    let second_generation = renames
        .iter()
        .filter(|rename| second_generation_ready < **rename && **rename < shutdown)
        .count();
    let unarmed_renames = renames
        .iter()
        .filter(|rename| did_change < **rename && **rename < second_generation_ready)
        .count();
    if first_generation_ready < did_change
        && did_change < second_generation_ready
        && unarmed_renames == 0
        && first_generation >= 2
        && second_generation >= 2
    {
        return Ok(());
    }
    Err(cstr!(
        "invalid cross-document readiness order in {}: ready={ready:?}, didChange={did_change}, unarmedRenames={unarmed_renames}, rename={renames:?}, shutdown={shutdown}",
        path.display()
    ).into())
}

fn find_all_bytes(haystack: &[u8], needle: &[u8]) -> Vec<usize> {
    haystack
        .windows(needle.len())
        .enumerate()
        .filter_map(|(index, window)| (window == needle).then_some(index))
        .collect()
}
