// Both callers check this witness against the original encounter order.
{
    let start = positions
        .get(query.start)
        .ok_or(GraphError::UnknownOp(query.start))?;
    let Some((root, rest)) = steps.split_first() else {
        return Err(CertificateError::Root);
    };
    if root.op != query.start || root.via.is_some() {
        return Err(CertificateError::Root);
    }
    let mut seen = vec![false; ops.len()];
    *seen
        .get_mut(start)
        .ok_or(GraphError::UnknownOp(query.start))? = true;
    for step in rest {
        let position = positions
            .get(step.op)
            .ok_or(GraphError::UnknownOp(step.op))?;
        if *seen.get(position).ok_or(GraphError::UnknownOp(step.op))? {
            return Err(CertificateError::Duplicate(step.op));
        }
        let witness = CertificateError::Witness { op: step.op };
        let ordinal = step.via.ok_or_else(|| witness.clone())?;
        let edge = edges.get(ordinal as usize).ok_or_else(|| witness.clone())?;
        let (from, to) = query.direction.endpoints(edge);
        let prior = positions.endpoint(ordinal, from)?;
        let prior = *seen.get(prior).ok_or(GraphError::UnknownOp(from))?;
        if !rule::justifies(
            prior,
            query.kinds.contains(edge.kind),
            to.index(),
            step.op.index(),
        ) {
            return Err(witness);
        }
        *seen
            .get_mut(position)
            .ok_or(GraphError::UnknownOp(step.op))? = true;
    }
    for (ordinal, edge) in edges.iter().enumerate() {
        let ordinal = ordinal as u32;
        let (from, to) = query.direction.endpoints(edge);
        let parent = *seen
            .get(positions.endpoint(ordinal, from)?)
            .ok_or(GraphError::UnknownOp(from))?;
        let child = *seen
            .get(positions.endpoint(ordinal, to)?)
            .ok_or(GraphError::UnknownOp(to))?;
        if !rule::closed(parent, query.kinds.contains(edge.kind), child) {
            return Err(CertificateError::OpenEdge(ordinal));
        }
    }
    Ok(())
}
