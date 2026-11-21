//! Join the original module's surface regions with its lowered render edits.

use super::{
    BooleanBindings, JsxComponent, JsxFile, JsxRefusal, JsxRoot, ModuleOptions, ModuleVisitor,
    PrologueIssue, RootContext, lower_component, walk_module,
};
use alloc::vec::Vec;
use oxc_parser::AdmittedProgram;
use oxc_semantic::Scoping;
use vize_l0::Allocator;
use vize_l1::embed::regions::{ModuleRegions, RegionError};
use vize_l2::module::{ComponentDef, ModuleDef, ModuleError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleLoweringError {
    Regions(RegionError),
    Component(JsxRefusal),
    Module(ModuleError),
}

/// A reusable module and the original prologue diagnostics in encounter order.
pub struct ModuleLowering<'a> {
    pub module: ModuleDef<'a>,
    pub issues: Vec<PrologueIssue<'a>>,
}

/// Retain the admitted source and its protected surface ranges once for every
/// downstream pass. No caller-supplied filename, source copy or span list is needed.
pub fn lower_module_def<'a>(
    allocator: &'a Allocator,
    input: &AdmittedProgram<'_, 'a>,
    scoping: Option<&Scoping>,
    options: ModuleOptions,
) -> Result<ModuleLowering<'a>, ModuleLoweringError> {
    let regions = ModuleRegions::from_program(input).map_err(ModuleLoweringError::Regions)?;
    lower_regions(allocator, input, scoping, options, regions)
}

/// Prepare the same definitions with their original callable and literal tokens.
#[cfg(feature = "component-sites")]
pub fn lower_module_with_component_sites<'a>(
    allocator: &'a Allocator,
    input: &AdmittedProgram<'_, 'a>,
    scoping: Option<&Scoping>,
    options: ModuleOptions,
) -> Result<ModuleLowering<'a>, ModuleLoweringError> {
    let regions =
        ModuleRegions::with_component_sites(input).map_err(ModuleLoweringError::Regions)?;
    lower_regions(allocator, input, scoping, options, regions)
}

fn lower_regions<'a>(
    allocator: &'a Allocator,
    input: &AdmittedProgram<'_, 'a>,
    scoping: Option<&Scoping>,
    options: ModuleOptions,
    regions: ModuleRegions<'a>,
) -> Result<ModuleLowering<'a>, ModuleLoweringError> {
    let program = input.program();
    let booleans = BooleanBindings::collect(program, scoping);
    let mut lower = Definitions {
        arena: allocator,
        file: JsxFile {
            source: input.source(),
            scoping,
            booleans: &booleans,
        },
        options,
        components: Ok(Vec::with_capacity(regions.roots().len())),
        issues: Vec::new(),
    };
    walk_module(program, &mut lower);
    let module = ModuleDef::checked(
        regions,
        lower.components.map_err(ModuleLoweringError::Component)?,
    )
    .map_err(ModuleLoweringError::Module)?;
    Ok(ModuleLowering {
        module,
        issues: lower.issues,
    })
}

// The admitted Program lends names and prologues for the module's lifetime.
// Write completed definitions directly into their final buffer.
struct Definitions<'a, 's> {
    arena: &'a Allocator,
    file: JsxFile<'a, 's>,
    options: ModuleOptions,
    components: Result<Vec<ComponentDef<'a>>, JsxRefusal>,
    issues: Vec<PrologueIssue<'a>>,
}

impl<'a> ModuleVisitor<'a> for Definitions<'a, '_> {
    fn prologue_issue(&mut self, issue: PrologueIssue<'a>) {
        self.issues.push(issue);
    }

    fn root(&mut self, root: JsxRoot<'_, 'a>, context: RootContext<'a>) {
        let Ok(components) = &mut self.components else {
            return;
        };
        let component = JsxComponent {
            name: context.name,
            exec: context.prologue.unwrap_or(self.options.default_exec),
        };
        match lower_component(
            self.arena,
            &self.file,
            self.options.dialect,
            component,
            root,
        ) {
            Ok(definition) => components.push(definition),
            Err(refusal) => self.components = Err(refusal),
        }
    }
}
