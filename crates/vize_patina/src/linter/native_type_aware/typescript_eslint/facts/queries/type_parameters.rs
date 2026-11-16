/// Only syntax shapes whose type-parameter usage cannot be decided from the
/// authored signature reach the checker: inferred returns and type predicates,
/// `typeof` returns, unannotated parameters and inferred class-member types.
fn type_parameter_queries(node: &LintNode, plan: &mut Planner) {
    let Some(items) = node
        .fields
        .get("__typeParameterUsageQueries")
        .and_then(Value::as_array)
    else {
        return;
    };
    for item in items {
        let (Some(start), Some(end), Some(kind)) = (
            field_u32(item, "start"),
            field_u32(item, "end"),
            item.get("kind").and_then(Value::as_str),
        ) else {
            continue;
        };
        let mut query = ExpressionQuery::at(start, end);
        if kind == "return" {
            query.function_returns = Some(false);
            query.predicates = true;
        }
        plan.push(query);
    }
}
