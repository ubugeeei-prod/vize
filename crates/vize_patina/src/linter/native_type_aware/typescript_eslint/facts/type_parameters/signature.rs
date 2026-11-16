/// The types typescript-eslint's `visitSignature` reads that the signature
/// does not spell, as checker queries: the inferred return (or the type
/// predicate it carries) and every parameter without an annotation, whose
/// type comes from its default value or its context. Written annotations are
/// already counted in the AST.
fn signature_queries(function: &Value, shadows: &[&str], queries: &mut Vec<Value>) {
    if needs_semantic_return(function)
        && let Some((start, end)) = node_range(function)
    {
        queries.push(json!({ "start": start, "end": end, "kind": "return", "shadows": shadows }));
    }
    let parameters = function.get("params").and_then(Value::as_array);
    for parameter in parameters.into_iter().flatten() {
        if let Some((start, end)) = unannotated_binding(parameter).and_then(node_range) {
            queries.push(json!({ "start": start, "end": end, "kind": "type", "shadows": shadows }));
        }
    }
}

/// The binding whose type the checker infers: `other` in `other = value`,
/// unless an annotation spells it.
fn unannotated_binding(parameter: &Value) -> Option<&Value> {
    let binding = match parameter.get("type").and_then(Value::as_str) {
        Some("TSParameterProperty") => return unannotated_binding(parameter.get("parameter")?),
        Some("AssignmentPattern") => parameter.get("left")?,
        _ => parameter,
    };
    (!has_type_annotation(binding)).then_some(binding)
}

/// A class property's initializer is typed through the property itself
/// (`value = undefined as T | undefined` is one use of `T`), so the type
/// references written inside it are not counted again. Function-valued
/// initializers are signatures of their own and keep their annotations.
fn is_property_initializer(owner_kind: Option<&str>, key: &str, child: &Value) -> bool {
    key == "value"
        && matches!(owner_kind, Some("PropertyDefinition" | "AccessorProperty"))
        && !is_function_like(child)
}
