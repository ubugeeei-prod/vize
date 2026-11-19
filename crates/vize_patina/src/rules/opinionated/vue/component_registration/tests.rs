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

#[test]
fn reads_class_component_decorator_registrations_with_inline_values() {
    let source = "@Component({ name: 'Setting', components: { VNodes: { functional: true, render: (h, ctx) => ctx.props.vnodes }, FooterBox } })\nexport default class Setting extends Vue {}";
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::ts()).parse();
    for tag in ["v-nodes", "VNodes", "footer-box"] {
        assert!(
            program_registers(&parsed.program, tag, RegistrationKeyPolicy::Runtime),
            "{tag}"
        );
    }
    assert!(!program_registers(
        &parsed.program,
        "a-divider",
        RegistrationKeyPolicy::Runtime
    ));
    let other = "@Injectable({ components: { VNodes } })\nexport default class Service {}";
    let parsed = Parser::new(&allocator, other, SourceType::ts()).parse();
    assert!(!program_registers(
        &parsed.program,
        "v-nodes",
        RegistrationKeyPolicy::Runtime
    ));
}
