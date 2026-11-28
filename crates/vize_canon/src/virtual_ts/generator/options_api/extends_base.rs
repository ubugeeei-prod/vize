//! Cast an `extends:` base imported from another SFC for Vue's mixin inference.

use oxc_ast::ast::{
    Expression, ImportDeclarationSpecifier, ImportOrExportKind, ObjectExpression, Program,
    Statement,
};
use vize_carton::String;

use super::option_expression_property;

/// The text after `typeof <base>` in the cast.
///
/// Vue reads an `extends` base's setup bindings from the return type of its
/// last `setup` overload. A generated SFC export intersects the authored
/// `DefineComponent` with `$VueComponentOptions`, whose `setup` member (kept
/// for `h()` prop inference) returns `any`, so the derived instance became
/// `any` and no inherited or own member was checked. When that last overload
/// returns `any`, the cast appends the overload before it, the authored
/// `setup`, as the new last one. Any other base, such as a `.ts` component or a
/// package component, has no such overload and keeps its type.
const EXTENDS_BASE_CAST: &str = " extends infer __B ? __B extends { setup?: infer __S } ? __S & {} extends { (this: void, props: any, ctx: any): infer __R; (this: void, props: any, ctx: any): infer __L } ? 0 extends 1 & __L ? __B & { setup?: (this: void, props: any, ctx: any) => __R } : __B : __B : __B : never)";

/// The `(offset, text)` cast to splice right after an `extends:` value that is
/// a value import from a `.vue` file.
pub(in crate::virtual_ts::generator) fn extends_base_assertion(
    program: &Program<'_>,
    options: &ObjectExpression<'_>,
) -> Option<(usize, String)> {
    let Expression::Identifier(base) = option_expression_property(options, "extends")? else {
        return None;
    };
    let name = base.name.as_str();
    if !imports_from_sfc(program, name) {
        return None;
    }
    let mut text = String::with_capacity(12 + name.len() + EXTENDS_BASE_CAST.len());
    text.push_str(" as (typeof ");
    text.push_str(name);
    text.push_str(EXTENDS_BASE_CAST);
    Some((base.span.end as usize, text))
}

fn imports_from_sfc(program: &Program<'_>, name: &str) -> bool {
    program.body.iter().any(|statement| {
        let Statement::ImportDeclaration(import) = statement else {
            return false;
        };
        import.import_kind == ImportOrExportKind::Value
            && import.source.value.as_str().ends_with(".vue")
            && import
                .specifiers
                .iter()
                .flatten()
                .any(|specifier| match specifier {
                    ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => {
                        specifier.local.name.as_str() == name
                    }
                    ImportDeclarationSpecifier::ImportSpecifier(specifier) => {
                        specifier.import_kind == ImportOrExportKind::Value
                            && specifier.local.name.as_str() == name
                    }
                    ImportDeclarationSpecifier::ImportNamespaceSpecifier(_) => false,
                })
    })
}

#[cfg(test)]
mod tests {
    use super::super::component_options_from_program;
    use super::{EXTENDS_BASE_CAST, extends_base_assertion};
    use oxc_allocator::Allocator;
    use oxc_parser::Parser;
    use oxc_span::SourceType;

    fn assertion(script: &str) -> Option<(usize, vize_carton::String)> {
        let allocator = Allocator::default();
        let parsed = Parser::new(&allocator, script, SourceType::ts()).parse();
        let options = component_options_from_program(&parsed.program)?;
        extends_base_assertion(&parsed.program, options)
    }

    #[test]
    fn casts_only_a_value_import_of_an_sfc() {
        let script = "import Base from './Base.vue'\nexport default { extends: Base, name: 'A' }";
        let (offset, text) = assertion(script).unwrap();
        assert_eq!(
            script.get(..offset).unwrap().rsplit(' ').next(),
            Some("Base")
        );
        assert_eq!(
            text.as_str(),
            [" as (typeof Base", EXTENDS_BASE_CAST].concat()
        );

        let named =
            "import { Base } from './Base.vue'\nexport default defineComponent({ extends: Base })";
        assert!(assertion(named).is_some());
        for script in [
            "import Base from './base'\nexport default { extends: Base }",
            "import type Base from './Base.vue'\nexport default { extends: Base }",
            "import * as Base from './Base.vue'\nexport default { extends: Base }",
            "const Base = { methods: {} }\nexport default { extends: Base }",
            "import Base from './Base.vue'\nexport default { mixins: [Base] }",
            "import Base from './Base.vue'\nexport default { extends: Base.extra }",
        ] {
            assert!(assertion(script).is_none(), "{script}");
        }
    }
}
