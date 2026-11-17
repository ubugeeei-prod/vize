use oxc_ast::ast::{Expression, TSType, TSTypeName};
use oxc_span::GetSpan;
use vize_carton::{CompactString, cstr};

pub(super) fn runtime_prop_type_from_ts_type(
    ts_type: &TSType<'_>,
    source: &str,
) -> Option<CompactString> {
    match ts_type {
        TSType::TSTypeReference(type_ref) => {
            let name = simple_type_name(&type_ref.type_name)?;
            matches!(name, "PropType" | "ReadonlyArray")
                .then(|| type_ref.type_arguments.as_ref())
                .flatten()
                .and_then(|args| args.params.first())
                .and_then(|ty| type_source(ty, source))
        }
        TSType::TSParenthesizedType(parenthesized) => {
            runtime_prop_type_from_ts_type(&parenthesized.type_annotation, source)
        }
        // Vue accepts constructors asserted as a factory signature, for
        // example `Object as () => Settings`, as a PropType-compatible
        // spelling. The callable is the constructor annotation here; the
        // authored prop value is its return type rather than `Function`.
        TSType::TSFunctionType(function) => {
            type_source(&function.return_type.type_annotation, source)
        }
        _ => None,
    }
}

fn type_source(ts_type: &TSType<'_>, source: &str) -> Option<CompactString> {
    let span = ts_type.span();
    source
        .get(span.start as usize..span.end as usize)
        .map(str::trim)
        .filter(|ty| !ty.is_empty())
        .map(CompactString::new)
}

fn simple_type_name<'a>(type_name: &'a TSTypeName<'_>) -> Option<&'a str> {
    match type_name {
        TSTypeName::IdentifierReference(id) => Some(id.name.as_str()),
        TSTypeName::QualifiedName(qualified) => Some(qualified.right.name.as_str()),
        _ => None,
    }
}

pub(super) fn runtime_ctor_type(name: &str) -> Option<&'static str> {
    match name {
        "String" => Some("string"),
        "Number" => Some("number"),
        "Boolean" => Some("boolean"),
        "Array" => Some("unknown[]"),
        // Vue's own `InferPropType` maps `ObjectConstructor` to
        // `Record<string, any>`; `unknown` here made every `v-for` over an
        // `Object` prop report TS18046 that vue-tsc does not (#4426 follow-up).
        "Object" => Some("Record<string, any>"),
        "Date" => Some("Date"),
        "Function" => Some("(...args: any[]) => any"),
        _ => None,
    }
}

/// The checker helper that types a runtime prop from its constructor value.
const VALUE_PROP_TYPE_HEAD: &str = "__RuntimePropCtor<typeof ";

/// Vue infers any other constructor (`Counter`, `ns.Counter`, a `PropType`
/// constant) from the value itself: an instance for `new`, a return value for
/// a call. Name the value and let the checker infer it, as the setup-side
/// `defineProps(...)` result does.
pub(super) fn runtime_value_prop_type(
    value: &Expression<'_>,
    reads_value: &mut bool,
) -> Option<CompactString> {
    let mut path = CompactString::default();
    push_value_path(value, &mut path)?;
    *reads_value = true;
    Some(cstr!("{VALUE_PROP_TYPE_HEAD}{path}>"))
}

fn push_value_path(value: &Expression<'_>, path: &mut CompactString) -> Option<()> {
    match value {
        Expression::Identifier(id) => path.push_str(id.name.as_str()),
        Expression::StaticMemberExpression(member) if !member.optional => {
            push_value_path(&member.object, path)?;
            path.push('.');
            path.push_str(member.property.name.as_str());
        }
        _ => return None,
    }
    Some(())
}

/// Root bindings named by [`runtime_value_prop_type`], e.g. `ns` in `ns.Counter`.
pub(in crate::script_parser) fn value_prop_type_roots(
    prop_type: &str,
) -> impl Iterator<Item = &str> {
    prop_type
        .split(VALUE_PROP_TYPE_HEAD)
        .skip(1)
        .filter_map(|rest| rest.split(['.', '>']).next())
}
