//! Factory sharing preserves the local ref registry and its demand boundary.
use crate::virtual_ts::{
    VirtualTsGenerationOptions, VirtualTsOptions, generate_virtual_ts_with_offsets_and_checks,
    helpers::SHARED_PREAMBLE_DTS,
    scope::{REF_INSTANCE_HELPERS, REF_INSTANCE_HELPERS_HOISTED},
};
use oxc_ast::ast::{
    Function, TSModuleDeclaration, TSModuleDeclarationName, TSTypeAliasDeclaration,
};
use oxc_ast_visit::{Visit, walk};
use oxc_parser::Parser;
use oxc_span::SourceType;
use oxc_syntax::scope::ScopeFlags;
use vize_carton::{Allocator, String, cstr};
use vize_croquis::{Analyzer, AnalyzerOptions};

const FACTORY: &str = include_str!("../../helpers/declarations/ref-instance-factory.d.ts.txt");
const FORWARD: &str =
    "  type $RefInstanceFactory<_C> = import(\"vize:virtual-ts\").$RefInstanceFactory<_C>;\n";

fn generate(script: &str, template: &str, hoisted: bool) -> String {
    let allocator = Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, template);
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    generate_virtual_ts_with_offsets_and_checks(
        &analyzer.finish(),
        Some(script),
        Some(&root),
        0,
        0,
        &VirtualTsOptions::default(),
        VirtualTsGenerationOptions {
            hoist_shared_preamble: hoisted,
            ..Default::default()
        },
    )
    .code
}

fn aliases<'s>(code: &'s str, name: &str) -> Vec<(std::string::String, &'s str)> {
    struct Declarations<'s, 'n> {
        source: &'s str,
        name: &'n str,
        owners: Vec<std::string::String>,
        whole: Vec<(std::string::String, &'s str)>,
    }
    impl<'a> Visit<'a> for Declarations<'_, '_> {
        fn visit_ts_type_alias_declaration(&mut self, declaration: &TSTypeAliasDeclaration<'a>) {
            if declaration.id.name.as_str() == self.name {
                self.whole.push((
                    self.owners.join("/"),
                    &self.source[declaration.span.start as usize..declaration.span.end as usize],
                ));
            }
            walk::walk_ts_type_alias_declaration(self, declaration);
        }
        fn visit_function(&mut self, function: &Function<'a>, flags: ScopeFlags) {
            let name = function
                .id
                .as_ref()
                .map_or("<anonymous>", |id| id.name.as_str());
            self.owners.push(cstr!("function:{name}").to_string());
            walk::walk_function(self, function, flags);
            self.owners.pop();
        }
        fn visit_ts_module_declaration(&mut self, module: &TSModuleDeclaration<'a>) {
            let name = match &module.id {
                TSModuleDeclarationName::Identifier(id) => id.name.as_str(),
                TSModuleDeclarationName::StringLiteral(literal) => literal.value.as_str(),
            };
            self.owners.push(cstr!("module:{name}").to_string());
            walk::walk_ts_module_declaration(self, module);
            self.owners.pop();
        }
    }
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, code, SourceType::ts()).parse();
    assert!(!parsed.panicked);
    assert_eq!(parsed.diagnostics.len(), 0, "{:?}", parsed.diagnostics);
    let mut declarations = Declarations {
        source: code,
        name,
        owners: Vec::new(),
        whole: Vec::new(),
    };
    declarations.visit_program(&parsed.program);
    declarations.whole
}

#[test]
fn sharing_changes_only_the_factory_body_in_the_local_helper_block() {
    let embedded_factory = cstr!("  {FACTORY}");
    assert_eq!(
        REF_INSTANCE_HELPERS
            .matches(embedded_factory.as_str())
            .count(),
        1
    );
    assert_eq!(REF_INSTANCE_HELPERS_HOISTED.matches(FORWARD).count(), 1);
    assert_eq!(
        REF_INSTANCE_HELPERS_HOISTED.replacen(FORWARD, &embedded_factory, 1),
        REF_INSTANCE_HELPERS,
    );
    assert_eq!(
        aliases(SHARED_PREAMBLE_DTS, "$RefInstanceFactory"),
        vec![("module:vize:virtual-ts".to_owned(), FACTORY.trim_end())],
    );
}

#[test]
fn instantiated_component_refs_select_the_forwarder_only_in_hoisted_programs() {
    let script = "import Child from './Child.vue'\nimport { useTemplateRef } from 'vue'\nconst child = useTemplateRef('child')\n";
    let template = r#"<Child ref="child" :value="1" />"#;
    for hoisted in [false, true] {
        let code = generate(script, template, hoisted);
        let expected = if hoisted {
            FORWARD.trim()
        } else {
            FACTORY.trim_end()
        };
        assert_eq!(
            aliases(&code, "$RefInstanceFactory"),
            vec![("function:__setup".to_owned(), expected)]
        );
        let instance = aliases(REF_INSTANCE_HELPERS, "$TemplateRefInstance");
        assert_eq!(instance.len(), 1);
        assert_eq!(instance[0].0, "");
        assert_eq!(
            aliases(&code, "$TemplateRefInstance"),
            vec![("function:__setup".to_owned(), instance[0].1)],
        );
    }
}

#[test]
fn unused_native_declared_foreign_and_typed_refs_do_not_gain_a_factory() {
    for (script, template) in [
        (
            "import Child from './Child.vue'",
            r#"<Child ref="child" :value="1" />"#,
        ),
        (
            "import { useTemplateRef } from 'vue'\nconst child = useTemplateRef('child')",
            r#"<div ref="child" />"#,
        ),
        (
            "import Child from './Child.vue'\nimport { useTemplateRef } from 'vue'\nconst child = useTemplateRef('child')",
            r#"<Child ref="child" />"#,
        ),
        (
            "import Child from './Child.vue'\nimport { useTemplateRef } from './foreign'\nconst child = useTemplateRef('child')",
            r#"<Child ref="child" :value="1" />"#,
        ),
        (
            "import Child from './Child.vue'\nimport { useTemplateRef } from 'vue'\nconst child = useTemplateRef<never>('child')",
            r#"<Child ref="child" :value="1" />"#,
        ),
    ] {
        for hoisted in [false, true] {
            assert_eq!(
                aliases(&generate(script, template, hoisted), "$RefInstanceFactory"),
                Vec::<(std::string::String, &str)>::new()
            );
        }
    }
}

#[test]
fn authored_module_factory_types_do_not_capture_the_local_forwarder() {
    let script = "import Child from './Child.vue'\nimport { useTemplateRef } from 'vue'\ntype $RefInstanceFactory = 'authored'\nconst child = useTemplateRef('child')\n";
    for hoisted in [false, true] {
        let code = generate(script, r#"<Child ref="child" :value="1" />"#, hoisted);
        let expected = if hoisted {
            FORWARD.trim()
        } else {
            FACTORY.trim_end()
        };
        assert_eq!(
            aliases(&code, "$RefInstanceFactory"),
            vec![
                (
                    std::string::String::new(),
                    "type $RefInstanceFactory = 'authored'"
                ),
                ("function:__setup".to_owned(), expected),
            ],
        );
    }
}

#[cfg(feature = "native")]
#[test]
fn batch_ref_factory_uses_pinned_generic_and_dom_inputs() {
    use crate::batch::VirtualProject;
    use std::{fs, path::Path};

    let inputs = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/_fixtures/differential/typechecker/shared-ref-factory");
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    fs::create_dir(root.join("src")).unwrap();
    for (source, target) in [
        ("Child.vue.txt", "src/Child.vue"),
        ("Probe.vue.txt", "src/Probe.vue"),
        ("tsconfig.json.txt", "tsconfig.json"),
    ] {
        fs::copy(inputs.join(source), root.join(target)).unwrap();
    }
    let child = root.join("src/Child.vue");
    let probe = root.join("src/Probe.vue");
    let mut project = VirtualProject::new(root).unwrap();
    project
        .register_paths(&[child.clone(), probe.clone()])
        .unwrap();
    assert_eq!(
        aliases(
            &project.find_by_original(&child).unwrap().content,
            "$RefInstanceFactory"
        ),
        Vec::<(std::string::String, &str)>::new()
    );
    assert_eq!(
        aliases(
            &project.find_by_original(&probe).unwrap().content,
            "$RefInstanceFactory"
        ),
        vec![("function:__setup".to_owned(), FORWARD.trim())]
    );
}
