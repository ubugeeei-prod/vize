use super::{RegistrationKeyPolicy, program_registers};
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;

#[test]
fn reads_direct_and_recognized_wrapped_options_api_registrations() {
    for source in [
        "export default { components: { Popover, 'algolia-search': Search } }",
        "export default defineComponent({ components: { Popover } })",
        "const Dialog = defineComponent({ components: { Popover } }); export default Dialog",
    ] {
        let allocator = Allocator::default();
        let parsed = Parser::new(&allocator, source, SourceType::ts()).parse();
        assert!(program_registers(
            &parsed.program,
            "popover",
            RegistrationKeyPolicy::Runtime
        ));
    }
}

#[test]
fn ignores_unrecognized_options_wrappers() {
    let allocator = Allocator::default();
    let parsed = Parser::new(
        &allocator,
        "export default create({ components: { Popover } })",
        SourceType::ts(),
    )
    .parse();
    assert!(!program_registers(
        &parsed.program,
        "popover",
        RegistrationKeyPolicy::Runtime
    ));
}

#[test]
fn separates_runtime_registration_from_authored_key_casing() {
    let allocator = Allocator::default();
    let parsed = Parser::new(
        &allocator,
        "export default { components: { svgIcon, Popover } }",
        SourceType::ts(),
    )
    .parse();
    assert!(program_registers(
        &parsed.program,
        "svg-icon",
        RegistrationKeyPolicy::Runtime
    ));
    assert!(!program_registers(
        &parsed.program,
        "svg-icon",
        RegistrationKeyPolicy::AuthoredPascal
    ));
    assert!(program_registers(
        &parsed.program,
        "popover",
        RegistrationKeyPolicy::Runtime
    ));
}
