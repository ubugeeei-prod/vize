macro_rules! require {
    ($condition:expr, $($message:tt)+) => {
        if !$condition {
            return Err(cstr!($($message)+));
        }
    };
    ($condition:expr) => {
        if !$condition {
            return Err(cstr!("law failed: {}", stringify!($condition)));
        }
    };
}
macro_rules! equal {
    ($left:expr, $right:expr $(,)?) => {
        if $left != $right {
            return Err(cstr!(
                "law failed: {} != {}",
                stringify!($left),
                stringify!($right)
            ));
        }
    };
}

use super::{FileArtifact, Observed};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_l0::{Allocator, String, cstr};
use vize_l2::file::{FileIssueKind, ScriptUnit, vue::ExposureIssueKind};
use vize_l2::lang::js::{
    CallEvent, DeclaredEvent, FileObserver, FileProducer, ProgramInput, ProgramScope, SetupIssue,
    SetupIssueKind, StatementEvent, VueSetup,
};
use vize_l2::resolution::ResolutionErrorKind;

mod annotations;
mod constants;
mod strict;
mod typescript;

fn checked<'o, 'f, 'a>(
    original: &'o Observed<'a>,
    file: &'f FileArtifact<'a>,
) -> Result<Result<VueSetup<'f, 'o, 'o, 'a>, SetupIssue>, &'static str> {
    Ok(VueSetup::checked(
        file,
        original.script()?,
        original
            .syntax
            .admitted_program()
            .ok_or("original Program")?,
    ))
}

#[test]
fn setup_eligibility_retains_original_source_and_all_actual_root_bindings() -> Result<(), String> {
    let arena = Allocator::default();
    let source = "<script setup>/* 雪 */; let count=1, yes=true; var nil=null, large=2n, text='🌸';;</script><template>{{ count }}</template>";
    let original = Observed::new(&arena, source)?;
    let file = original.file(&arena)?;
    let setup = checked(&original, &file)?.map_err(|_| "genuine setup")?;
    require!(core::ptr::eq(setup.exposure().file(), &file));
    require!(core::ptr::eq(setup.source().root_source(), source));
    require!(core::ptr::eq(
        setup.source().source(),
        original.script()?.block().source()
    ));
    equal!(setup.bindings().count(), 5);
    equal!(original.syntax.comments().count(), 1);
    for (binding, name) in setup
        .bindings()
        .zip(["count", "yes", "nil", "large", "text"])
    {
        require!(core::ptr::eq(binding.file(), &file));
        let declaration = binding.declaration().ok_or("actual binding")?;
        equal!(declaration.name.as_str(), name);
        equal!(declaration.scope, setup.exposure().scope());
        equal!(declaration.script_unit(), Some(setup.exposure().unit()));
        require!(declaration.is_direct_program());
    }
    Ok(())
}

#[test]
fn neutral_complete_files_do_not_certify_the_whole_setup_statement_family() -> Result<(), String> {
    let arena = Allocator::default();
    for script in [
        "let value=1; 42;",
        "42; let value=1;",
        "const original=1; const value=original;",
        "let value=1; var copy=value;",
        "var value;",
        "let value=1; value=2;",
        "function hidden(){} let value=1;",
        "'use strict'; let value=1;",
        "#!/usr/bin/env node\nlet value=1;",
    ] {
        let source = cstr!("<script setup>{script}</script>");
        let original = Observed::new(&arena, &source)?;
        let file = original.file(&arena)?;
        require!(file.is_complete(), "{script}");
        require!(
            original.view(&file)?.is_ok(),
            "membership remains separate: {script}"
        );
        require!(
            matches!(checked(&original, &file)?, Err(issue)
            if issue.kind == SetupIssueKind::UnsupportedSyntax),
            "{script}"
        );
        equal!(original.syntax.diagnostics().count(), 0);
    }
    Ok(())
}

#[test]
fn unsupported_script_shapes_never_supply_a_setup_capability() -> Result<(), String> {
    let arena = Allocator::default();
    for script in [
        "let value={};",
        "const value={};",
        "const value=[];",
        "const value=/x/;",
        "const value=`text`;",
        "const value=-1;",
        "const value=()=>1;",
        "const {value}={value:1};",
        "let value=[];",
        "let value=/x/;",
        "let value=`text`;",
        "let value=-1;",
        "let value=this;",
        "let value=()=>1;",
        "let value=new Number(1);",
        "let value=import('x');",
        "function tag(){} let value=tag`x`;",
        "let value=defineProps();",
        "import value from 'x';",
        "export let value=1;",
        "let {value}={value:1};",
        "let [value]=[1];",
        "{var hidden=1;} let value=1;",
        "let value=1; var value=2;",
        "function hidden(value){let value=1;} let visible=1;",
        "let __props=1;",
        "let __expose=1;",
        "let __returned__=1;",
        "let __proto__=1;",
    ] {
        let source = cstr!("<script setup>{script}</script>");
        let original = Observed::new(&arena, &source)?;
        let file = original.file(&arena)?;
        require!(checked(&original, &file)?.is_err(), "{script}");
        require!(core::ptr::eq(file.artifact().source(), source.as_str()));
        require!(original.syntax.program().is_some());
    }
    Ok(())
}

#[test]
fn genuine_empty_statements_admit_but_empty_or_ordinary_bodies_refuse() -> Result<(), String> {
    let arena = Allocator::default();
    let original = Observed::new(&arena, "<script setup>/* retained */;;;</script>")?;
    let file = original.file(&arena)?;
    let setup = checked(&original, &file)?.map_err(|_| "actual empty statements")?;
    equal!(setup.bindings().count(), 0);
    equal!(original.syntax.comments().count(), 1);
    for source in [
        "<script setup>/* retained */</script>",
        "<script>let value=1;</script>",
    ] {
        let original = Observed::new(&arena, source)?;
        let file = original.file(&arena)?;
        require!(file.is_complete());
        require!(checked(&original, &file)?.is_err(), "{source}");
    }
    Ok(())
}

#[test]
fn foreign_equal_sources_or_a_second_original_program_cannot_supply_setup() -> Result<(), String> {
    let arena = Allocator::default();
    let source = String::from("<script setup>let value=1;</script>");
    let copy = source.clone();
    let original = Observed::new(&arena, &source)?;
    let foreign = Observed::new(&arena, &copy)?;
    let file = original.file(&arena)?;
    require!(matches!(checked(&foreign, &file)?, Err(issue)
        if issue.kind == SetupIssueKind::Exposure(ExposureIssueKind::Source)));
    let script = original.script()?;
    let second = Parser::new(&arena, script.block().source(), SourceType::mjs()).parse_observed();
    require!(
        matches!(VueSetup::checked(&file, script, second.admitted().ok_or("second admission")?), Err(issue)
        if issue.kind == SetupIssueKind::Exposure(ExposureIssueKind::ProgramOrigin))
    );
    require!(checked(&original, &file)?.is_ok());
    Ok(())
}

#[test]
fn caller_selected_module_scope_does_not_replace_the_authentic_setup_root() -> Result<(), String> {
    let arena = Allocator::default();
    let original = Observed::new(&arena, "<script setup>let value=1;</script>")?;
    let script = original.script()?;
    let mut producer =
        FileProducer::new(&arena, original.descriptor.source()).map_err(|_| "file")?;
    producer
        .program(
            ProgramInput::checked(
                original.syntax.admitted_program().ok_or("Program")?,
                script.block(),
                script.container_index(),
            )
            .map_err(|_| "input")?,
            ProgramScope::Module,
        )
        .map_err(|_| "unit")?;
    let file = producer.finish().map_err(|_| "artifact")?;
    require!(file.is_complete());
    require!(original.view(&file)?.is_ok());
    require!(
        matches!(checked(&original, &file)?, Err(issue) if issue.kind == SetupIssueKind::Scope)
    );
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Point {
    Unit,
    Statement,
    Declared,
}
struct Interrupt(Point);
impl Interrupt {
    fn observe(&self, point: Point) {
        if self.0 == point {
            std::panic::resume_unwind(Box::new("actual setup walk interruption"));
        }
    }
}
impl<'a> FileObserver<'a> for Interrupt {
    type Checkpoint = ();
    fn unit(&mut self, _: &ScriptUnit) {
        self.observe(Point::Unit);
    }
    fn statement(&mut self, _: StatementEvent<'_, 'a>) {
        self.observe(Point::Statement);
    }
    fn declared(&mut self, _: DeclaredEvent<'_, 'a>) {
        self.observe(Point::Declared);
    }
    fn checkpoint(&self) {}
    fn call(&mut self, _: CallEvent<'_, 'a>) -> Result<(), ResolutionErrorKind> {
        Ok(())
    }
    fn rollback(&mut self, _: ()) {}
}

#[test]
fn every_actual_setup_callback_interruption_refuses_sealing_with_original_owners()
-> Result<(), String> {
    let arena = Allocator::default();
    for body in [
        "let first=1;var after=2;",
        "const first=1;let mutable=2;var after=3;",
    ] {
        let source = cstr!("<script setup>/* kept */{body}</script>");
        let original = Observed::new(&arena, &source)?;
        let script = original.script()?;
        for point in [Point::Unit, Point::Statement, Point::Declared] {
            let mut producer =
                FileProducer::new(&arena, original.descriptor.source()).map_err(|_| "file")?;
            let mut observer = Interrupt(point);
            let interrupted = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                producer
                    .program_observed(
                        ProgramInput::checked(
                            original.syntax.admitted_program().ok_or("Program")?,
                            script.block(),
                            script.container_index(),
                        )
                        .map_err(|_| "input")?,
                        ProgramScope::Nested,
                        &mut observer,
                    )
                    .map_err(|_| "unit")
            }));
            let payload = interrupted.err().ok_or("actual interruption")?;
            equal!(
                payload.downcast_ref::<&str>(),
                Some(&"actual setup walk interruption")
            );
            let file = producer.finish().map_err(|_| "artifact")?;
            require!(!file.is_complete(), "{point:?}");
            require!(file.issues().is_empty());
            equal!(file.interrupted_programs().count(), 1);
            let unit = file.units().first().ok_or("retained unit")?;
            equal!(
                unit.interruption().ok_or("interrupted row")?.kind,
                FileIssueKind::InterruptedProgram
            );
            equal!(
                file.bindings().count(),
                usize::from(point == Point::Declared)
            );
            require!(matches!(checked(&original, &file)?, Err(issue)
            if issue.kind == SetupIssueKind::Exposure(ExposureIssueKind::IncompleteFile)));
            equal!(original.syntax.comments().count(), 1);
        }
    }
    Ok(())
}
