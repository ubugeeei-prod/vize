//! Actual first-unit interruption must allocate exactly the unwind baseline.
//! Each measured mode runs in a separate fresh process with a preowned payload.

use oxc_parser::Parser;
use oxc_span::SourceType;
use std::alloc::{GlobalAlloc, Layout, System};
use std::any::Any;
use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use vize_l0::{Allocator, SourceRoot, Span};
use vize_l2::artifact::ComponentFactory;
use vize_l2::expr::JsExpr;
use vize_l2::file::{Declaration, ScriptUnit, TemplatePolicy, TemplateScope};
use vize_l2::lang::js::{
    CallEvent, DeclaredEvent, FileObserver, FileProducer, ProgramInput, ProgramScope,
    StatementEvent,
};
use vize_l2::resolution::ResolutionErrorKind;

static ENABLED: AtomicBool = AtomicBool::new(false);
static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);
struct Counter;
fn count() {
    if ENABLED.load(Ordering::Relaxed) {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
    }
}
// SAFETY: every request is delegated unchanged to the same System allocator.
unsafe impl GlobalAlloc for Counter {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        count();
        // SAFETY: forwarding the caller's valid allocation request.
        unsafe { System.alloc(layout) }
    }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        count();
        // SAFETY: forwarding the caller's valid zeroed-allocation request.
        unsafe { System.alloc_zeroed(layout) }
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        count();
        // SAFETY: ptr/layout came from this same delegated System allocator.
        unsafe { System.realloc(ptr, layout, size) }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // SAFETY: ptr/layout came from this same delegated System allocator.
        unsafe { System.dealloc(ptr, layout) }
    }
}
#[global_allocator]
static COUNTER: Counter = Counter;

struct Observer {
    payload: Option<Box<dyn Any + Send>>,
    seen: bool,
}
impl<'a> FileObserver<'a> for Observer {
    type Checkpoint = ();
    fn unit(&mut self, _: &ScriptUnit) {
        self.seen = true;
        if let Some(payload) = self.payload.take() {
            ENABLED.store(true, Ordering::Relaxed);
            std::panic::resume_unwind(payload);
        }
    }
    fn statement(&mut self, _: StatementEvent<'_, 'a>) {}
    fn declared(&mut self, _: DeclaredEvent<'_, 'a>) {}
    fn checkpoint(&self) {}
    fn call(&mut self, _: CallEvent<'_, 'a>) -> Result<(), ResolutionErrorKind> {
        Ok(())
    }
    fn rollback(&mut self, _: ()) {}
}

fn mode(guarded: bool) -> Result<usize, &'static str> {
    // Identical original source/parser/producer setup precedes either window.
    let arena = Allocator::default();
    let source = "const value = 1;";
    let parsed = Parser::new(&arena, source, SourceType::mjs()).parse_observed();
    let block = SourceRoot::new(source)
        .map_err(|_| "source root")?
        .whole_block();
    let input = ProgramInput::checked(
        parsed.admitted().ok_or("original parser admission")?,
        block,
        5,
    )
    .map_err(|_| "original block")?;
    let mut producer = FileProducer::new(&arena, source).map_err(|_| "file owner")?;
    let payload: Box<dyn Any + Send> = Box::new("preowned interruption");
    ALLOCATIONS.store(0, Ordering::Relaxed);
    if guarded {
        let mut observer = Observer {
            payload: Some(payload),
            seen: false,
        };
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            producer.program_observed(input, ProgramScope::Module, &mut observer)
        }));
        ENABLED.store(false, Ordering::Relaxed);
        let measured = ALLOCATIONS.load(Ordering::Relaxed);
        if result.is_ok() || !observer.seen {
            return Err("actual first-unit interruption missing");
        }
        let file = producer.finish().map_err(|_| "canonical owner")?;
        if file.is_complete()
            || file.units().first().map(|unit| unit.id.index()) != Some(5)
            || !core::ptr::eq(file.artifact().source(), source)
        {
            return Err("interrupted original owner not retained");
        }
        Ok(measured)
    } else {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            ENABLED.store(true, Ordering::Relaxed);
            std::panic::resume_unwind(payload);
        }));
        ENABLED.store(false, Ordering::Relaxed);
        let measured = ALLOCATIONS.load(Ordering::Relaxed);
        if result.is_ok() {
            return Err("baseline interruption missing");
        }
        Ok(measured)
    }
}

fn fresh(mode: &str) -> Result<usize, &'static str> {
    let executable = std::env::current_exe().map_err(|_| "actual executable")?;
    let output = std::process::Command::new(executable)
        .arg(mode)
        .output()
        .map_err(|_| "fresh process")?;
    if !output.status.success() {
        return Err("fresh measured mode failed");
    }
    std::str::from_utf8(&output.stdout)
        .map_err(|_| "measurement bytes")?
        .trim()
        .parse()
        .map_err(|_| "actual allocation count")
}

#[derive(Clone, Copy)]
struct InterruptPolicy<'s>(&'s RefCell<Option<Box<dyn Any + Send>>>);
impl TemplatePolicy for InterruptPolicy<'_> {
    fn visible(self, _: &Declaration) -> bool {
        if let Some(payload) = self.0.borrow_mut().take() {
            ENABLED.store(true, Ordering::Relaxed);
            std::panic::resume_unwind(payload);
        }
        true
    }
}
fn template_mode(guarded: bool) -> Result<usize, &'static str> {
    let arena = Allocator::default();
    let source = "const value = 1;\nvalue";
    let end = source.find('\n').ok_or("script boundary")?;
    let block = SourceRoot::new(source)
        .map_err(|_| "source root")?
        .block(source.get(..end).ok_or("script slice")?, 0)
        .map_err(|_| "script block")?;
    let parsed = Parser::new(&arena, block.source(), SourceType::mjs()).parse_observed();
    let input = ProgramInput::checked(parsed.admitted().ok_or("parser admission")?, block, 5)
        .map_err(|_| "original program")?;
    let mut producer = FileProducer::new(&arena, source).map_err(|_| "file owner")?;
    producer
        .program(input, ProgramScope::Nested)
        .map_err(|_| "completed script")?;
    let span = Span::new(end as u32 + 1, source.len() as u32);
    let expression = JsExpr::parse_in(
        &arena,
        source.get(end + 1..).ok_or("expression slice")?,
        span,
    )
    .map_err(|_| "retained expression")?;
    let payload: Box<dyn Any + Send> = Box::new("preowned template interruption");
    ALLOCATIONS.store(0, Ordering::Relaxed);
    if guarded {
        let payload = RefCell::new(Some(payload));
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut region = producer
                .template_region(TemplateScope::LastUnit, InterruptPolicy(&payload))
                .map_err(|_| "actual template region")?;
            let mut walk = region.walk(span).map_err(|_| "actual whole driver")?;
            let result = walk
                .interpolation(expression, span)
                .map_err(|_| "actual interpolation");
            walk.complete().map_err(|_| "actual whole interruption")?;
            result
        }));
        ENABLED.store(false, Ordering::Relaxed);
        let measured = ALLOCATIONS.load(Ordering::Relaxed);
        if result.is_ok() {
            return Err("actual pre-mint policy interruption missing");
        }
        let file = producer.finish().map_err(|_| "retained canonical owner")?;
        if file.is_complete()
            || file.template_interruption().map(|issue| issue.span) != Some(span)
            || file.artifact().node_count() != 0
            || !core::ptr::eq(file.artifact().source(), source)
        {
            return Err("interrupted template owner not retained");
        }
        Ok(measured)
    } else {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            ENABLED.store(true, Ordering::Relaxed);
            std::panic::resume_unwind(payload);
        }));
        ENABLED.store(false, Ordering::Relaxed);
        let measured = ALLOCATIONS.load(Ordering::Relaxed);
        if result.is_ok() {
            return Err("template baseline interruption missing");
        }
        Ok(measured)
    }
}

const CASE: &str = "first_unit_interruption_adds_no_allocation";
const TEMPLATE_CASE: &str = "pre_mint_template_interruption_adds_no_allocation";

fn main() -> Result<(), &'static str> {
    let mut args = std::env::args().skip(1).peekable();
    if matches!(
        args.peek().map(|arg| arg.as_str()),
        Some("baseline" | "guarded" | "template-baseline" | "template-guarded")
    ) {
        let private_mode = args.next().ok_or("private mode")?;
        if args.next().is_some() {
            return Err("unexpected private measurement argument");
        }
        let count = match private_mode.as_str() {
            "baseline" => mode(false)?,
            "guarded" => mode(true)?,
            "template-baseline" => template_mode(false)?,
            "template-guarded" => template_mode(true)?,
            _ => return Err("unsupported private mode"),
        };
        println!("{count}");
        return Ok(());
    }
    let (mut list, mut ignored, mut exact) = (false, false, false);
    let mut filter = None;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--list" => list = true,
            "--ignored" => ignored = true,
            "--exact" => exact = true,
            "--nocapture" => {}
            "--format" if args.next().as_deref() == Some("terse") => {}
            value if !value.starts_with('-') && filter.is_none() => filter = Some(arg),
            _ => return Err("unsupported harness argument"),
        }
    }
    if ignored {
        return Ok(());
    }
    for case in [CASE, TEMPLATE_CASE] {
        let selected = filter.as_ref().is_none_or(|filter| {
            if exact {
                filter == case
            } else {
                case.contains(filter)
            }
        });
        if !selected {
            continue;
        }
        if list {
            println!("{case}: test");
            continue;
        }
        let (baseline, guarded) = if case == CASE {
            (fresh("baseline")?, fresh("guarded")?)
        } else {
            (fresh("template-baseline")?, fresh("template-guarded")?)
        };
        println!("{case}: unwind allocations baseline={baseline}, guarded={guarded}");
        if guarded != baseline {
            return Err("guard adds an allocation while unwinding");
        }
    }
    Ok(())
}
