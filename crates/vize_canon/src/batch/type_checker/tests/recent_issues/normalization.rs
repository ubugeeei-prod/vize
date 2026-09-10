pub(crate) fn normalize_component_check_props_tail(message: &str) -> std::string::String {
    const START: &str = "__VizeComponentCheckProps<Props, ";
    const STABLE_TYPE: &str = "__VizeComponentCheckProps<Props, __VizeFallthroughAttrs>";
    let mut output = std::string::String::with_capacity(message.len());
    let mut rest = message;
    while let Some(start) = rest.find(START) {
        output.push_str(&rest[..start]);
        let from_type = &rest[start..];
        let Some(end) = from_type.find("'.") else {
            output.push_str(from_type);
            return output;
        };
        output.push_str(STABLE_TYPE);
        rest = &from_type[end..];
    }
    output.push_str(rest);
    output
}
