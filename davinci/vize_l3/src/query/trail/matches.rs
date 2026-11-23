// Enumerate the ordered source relationships independently of the adjacency index.
{
    let mut at = 0;
    for (first, left) in edges.iter().enumerate() {
        let (from, middle) = query.first.direction.endpoints(left);
        if from != query.start || !query.first.kinds.contains(left.kind) {
            continue;
        }
        for (second, right) in edges.iter().enumerate() {
            let (through, end) = query.second.direction.endpoints(right);
            if first == second || middle != through || !query.second.kinds.contains(right.kind) {
                continue;
            }
            let first_edge = first as u32;
            let second_edge = second as u32;
            let expected = TwoHopMatch {
                first_edge,
                second_edge,
                middle,
                end,
            };
            if rows.get(at) != Some(&expected) {
                return Err(TrailError::Mismatch {
                    at,
                    first_edge,
                    second_edge,
                });
            }
            at += 1;
        }
    }
    if rows.len() != at {
        return Err(TrailError::Extra { at });
    }
    Ok(())
}
