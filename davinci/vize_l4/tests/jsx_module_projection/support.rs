use oxc_ast::ast::{JSXElement, JSXFragment, Program};
use oxc_ast_visit::Visit;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use vize_l0::Allocator;
use vize_l1_to_l2::framework::vue::jsx::{
    BooleanBindings, JsxComponent, JsxFile, JsxRefusal, JsxRoot, VIZE, lower_component,
};
use vize_l2::component::{ComponentDef, Exec};

/// One parsed module and the L2 definitions of its outermost roots.
pub struct Module<'a> {
    pub program: Program<'a>,
    pub roots: Vec<ComponentDef<'a>>,
}

/// Parse `source` once and lower each outermost root with Vize semantics.
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
    let booleans = BooleanBindings::collect(&parsed.program, Some(&scoping));
    let file = JsxFile {
        source,
        scoping: Some(&scoping),
        booleans: &booleans,
    };
    let mut roots = Roots {
        arena,
        file: &file,
        lowered: Ok(Vec::new()),
    };
    roots.visit_program(&parsed.program);
    let roots = roots.lowered?;
    Ok(Module {
        program: parsed.program,
        roots,
    })
}

struct Roots<'f, 'a, 's> {
    arena: &'a Allocator,
    file: &'f JsxFile<'a, 's>,
    lowered: Result<Vec<ComponentDef<'a>>, JsxRefusal>,
}

impl Roots<'_, '_, '_> {
    fn lower(&mut self, root: JsxRoot<'_, '_>) {
        let component = JsxComponent {
            name: None,
            exec: Exec::ReRender,
        };
        if let Ok(roots) = &mut self.lowered {
            match lower_component(self.arena, self.file, &VIZE, component, root) {
                Ok(def) => roots.push(def),
                Err(refusal) => self.lowered = Err(refusal),
            }
        }
    }
}

impl<'x> Visit<'x> for Roots<'_, '_, '_> {
    fn visit_jsx_element(&mut self, element: &JSXElement<'x>) {
        self.lower(JsxRoot::Element(element));
    }

    fn visit_jsx_fragment(&mut self, fragment: &JSXFragment<'x>) {
        self.lower(JsxRoot::Fragment(fragment));
    }
}
