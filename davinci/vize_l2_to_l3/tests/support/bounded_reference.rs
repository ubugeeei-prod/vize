use vize_l3::{
    L3Ir,
    op::OpId,
    query::{BoundedTrailQuery, Direction},
};

// Reference DFS reads the original table for every endpoint. Its recursive stack
// is bounded by the short query length used in the fixtures and corpus.
// Ordinals order the alternatives, including parallel relationships.
pub(crate) fn scan(
    ir: &L3Ir<'_>,
    q: BoundedTrailQuery,
    at: OpId,
    prefix: &mut Vec<u32>,
    rows: &mut Vec<(OpId, Vec<u32>)>,
) {
    if (q.min_hops as usize..=q.max_hops as usize).contains(&prefix.len()) {
        rows.push((at, prefix.clone()));
    }
    if prefix.len() >= q.max_hops as usize {
        return;
    }
    for (i, e) in ir.edges.iter().enumerate() {
        let (from, to) = match q.step.direction {
            Direction::Forward => (e.from, e.to),
            Direction::Reverse => (e.to, e.from),
        };
        if from == at && q.step.kinds.contains(e.kind) && !prefix.contains(&(i as u32)) {
            prefix.push(i as u32);
            scan(ir, q, to, prefix, rows);
            prefix.pop();
        }
    }
}
