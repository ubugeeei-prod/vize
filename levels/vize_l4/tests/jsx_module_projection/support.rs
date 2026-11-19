use oxc_ast::ast::Program;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use vize_l0::Allocator;
use vize_l1_to_l2::framework::vue::jsx::{JsxRefusal, ModuleOptions, VIZE, lower_module};
use vize_l2::component::{ComponentDef, Exec};

/// One parsed module and the L2 definitions of its outermost roots.
pub struct Module<'a> {
    pub program: Program<'a>,
    pub roots: Vec<ComponentDef<'a>>,
}

/// Parse `source` once and lower its module with Vize semantics.
pub fn lower<'a>(
    arena: &'a Allocator,
    source: &'a str,
    tsx: bool,
) -> Result<Module<'a>, JsxRefusal> {
    let source_type = if tsx {
        SourceType::tsx()
    } else {
        SourceType::jsx()
    }
    .with_module(true);
    let parsed = Parser::new(arena.as_oxc(), source, source_type).parse();
    assert_eq!(parsed.diagnostics.len(), 0, "{source}");
    let scoping = SemanticBuilder::new()
        .build(&parsed.program)
        .semantic
        .into_scoping();
    let options = ModuleOptions {
        dialect: &VIZE,
        default_exec: Exec::ReRender,
    };
    let roots =
        lower_module(arena, source, &parsed.program, Some(&scoping), options).into_components()?;
    Ok(Module {
        program: parsed.program,
        roots,
    })
}
