//! The TS-48 echo guest: answers `lower-block` with the committed goldens
//! for the block whose source it recognizes, byte for byte, spins or hoards
//! memory on the two limit probes, and traps on any other block. It imports
//! no host function — no WASI — so it is `no_std` with its own allocator,
//! `cabi_realloc` and panic handler.

#![no_std]

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

wit_bindgen::generate!({
    path: "../../../../vize_extension_sdk/wit",
    world: "input-dialect",
});

use exports::vize::contracts::handshake::{Capability, Guest as Handshake};
use exports::vize::contracts::input_lowering::{Guest as Lowering, LoweredBlock, SourceBlock};
use vize::contracts::types::{
    Diagnostic, DiagnosticPart, Page, PartKind, Severity, Span, Stage, Witness,
};

struct Golden {
    source: &'static str,
    surface: &'static str,
    semantic: &'static str,
    diagnostics: &'static str,
}

macro_rules! golden {
    ($name:literal) => {
        Golden {
            source: include_str!(concat!("../../../fixtures/golden/", $name, ".block")),
            surface: include_str!(concat!("../../../fixtures/golden/", $name, ".s1.folio")),
            semantic: include_str!(concat!("../../../fixtures/golden/", $name, ".s2.folio")),
            diagnostics: include_str!(concat!("../../../fixtures/golden/", $name, ".diagnostics")),
        }
    };
}

const GOLDENS: &[Golden] = &[
    golden!("attributes"),
    golden!("control-flow"),
    golden!("recovery"),
    golden!("unicode"),
    // The empty block: hand-written diagnostics in every shape the WIT
    // `diagnostic` record can take.
    golden!("probe"),
];

#[cfg(not(feature = "offer-protocol-2"))]
const PROTOCOL_VERSION: u32 = 1;
#[cfg(feature = "offer-protocol-2")]
const PROTOCOL_VERSION: u32 = 2;

struct Echo;

impl Handshake for Echo {
    fn get_capability() -> Capability {
        Capability {
            protocol_version: PROTOCOL_VERSION,
            features: ["lang:html", "s1-page@1", "s2-page@1"]
                .into_iter()
                .map(String::from)
                .collect(),
        }
    }
}

impl Lowering for Echo {
    fn lower_block(block: SourceBlock) -> LoweredBlock {
        // The P6-3 limit probes: a runaway loop and a memory hoard.
        match block.source.as_str() {
            "<!-- spin -->" => loop {
                core::hint::black_box(());
            },
            "<!-- hoard -->" => {
                let mut hoard: Vec<Vec<u8>> = Vec::new();
                loop {
                    // Reserve without touching: memory grows, fuel barely moves.
                    hoard.push(Vec::with_capacity(1 << 20));
                    core::hint::black_box(&hoard);
                }
            }
            _ => {}
        }
        let Some(golden) = GOLDENS.iter().find(|golden| golden.source == block.source) else {
            core::arch::wasm32::unreachable()
        };
        LoweredBlock {
            surface: Page {
                schema_version: 1,
                text: String::from(golden.surface),
            },
            semantic: Page {
                schema_version: 1,
                text: String::from(golden.semantic),
            },
            diagnostics: diagnostics(golden.diagnostics),
        }
    }
}

export!(Echo);

/// Read the golden diagnostics lines (`tests/support/mod.rs` writes
/// them): `diagnostic <severity> <stage> <start>:<end> <message>`, then
/// that diagnostic's `part <kind> <start>:<end> <message>` and
/// `witness legacy-exempt <producer>` lines.
fn diagnostics(text: &str) -> Vec<Diagnostic> {
    let mut out: Vec<Diagnostic> = Vec::new();
    for line in text.lines() {
        let (head, rest) = line.split_once(' ').unwrap_or((line, ""));
        match head {
            "diagnostic" => {
                let (severity, rest) = word(rest);
                let (stage, rest) = word(rest);
                let (span, message) = span(rest);
                out.push(Diagnostic {
                    severity: match severity {
                        "error" => Severity::Error,
                        "warning" => Severity::Warning,
                        "info" => Severity::Info,
                        _ => Severity::Hint,
                    },
                    stage: match stage {
                        "source" => Stage::Source,
                        "surface" => Stage::Surface,
                        "semantic" => Stage::Semantic,
                        "lowered" => Stage::Lowered,
                        _ => Stage::Emit,
                    },
                    span,
                    message: String::from(message),
                    parts: Vec::new(),
                    witness: None,
                });
            }
            "part" => {
                let (kind, rest) = word(rest);
                let (span, message) = span(rest);
                let last = out.last_mut().expect("a part follows its diagnostic");
                last.parts.push(DiagnosticPart {
                    kind: match kind {
                        "primary" => PartKind::Primary,
                        "secondary" => PartKind::Secondary,
                        "help" => PartKind::Help,
                        _ => PartKind::Suggestion,
                    },
                    span,
                    message: String::from(message),
                });
            }
            _ => {
                let producer = rest.strip_prefix("legacy-exempt ").unwrap_or(rest);
                let last = out.last_mut().expect("a witness follows its diagnostic");
                last.witness = Some(Witness::LegacyExempt(String::from(producer)));
            }
        }
    }
    out
}

fn word(text: &str) -> (&str, &str) {
    text.split_once(' ').unwrap_or((text, ""))
}

fn span(text: &str) -> (Span, &str) {
    let (offsets, rest) = word(text);
    let (start, end) = offsets.split_once(':').unwrap_or((offsets, offsets));
    let number = |digits: &str| digits.parse::<u32>().unwrap_or(u32::MAX);
    (
        Span {
            start: number(start),
            end: number(end),
        },
        rest,
    )
}

/// `memcmp` for a guest that links no C library (`wasm32` has no compare
/// instruction, so `compiler_builtins` leaves it to libc). Volatile reads
/// keep the optimizer from recognizing this loop as a call to itself.
///
/// # Safety
///
/// `a` and `b` are valid for `len` bytes (the C contract).
// Deliberate: no C library links into this guest, so this is the only
// `memcmp` in the component, not a clash with one.
#[allow(suspicious_runtime_symbol_definitions)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn memcmp(a: *const u8, b: *const u8, len: usize) -> i32 {
    for index in 0..len {
        // SAFETY: both ranges are valid for `len` bytes.
        let (x, y) = unsafe {
            (
                core::ptr::read_volatile(a.add(index)),
                core::ptr::read_volatile(b.add(index)),
            )
        };
        if x != y {
            return i32::from(x) - i32::from(y);
        }
    }
    0
}

/// `bcmp`, the equality-only form LLVM emits for `==` on byte slices.
///
/// # Safety
///
/// As [`memcmp`].
#[allow(suspicious_runtime_symbol_definitions)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bcmp(a: *const u8, b: *const u8, len: usize) -> i32 {
    // SAFETY: forwarded under the same contract.
    unsafe { memcmp(a, b, len) }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    core::arch::wasm32::unreachable()
}

#[global_allocator]
static ALLOCATOR: dlmalloc::GlobalDlmalloc = dlmalloc::GlobalDlmalloc;

/// The canonical ABI's allocation entry point. On `wasm32-wasip2` the C
/// library normally provides it; a `no_std` guest links no C library.
///
/// # Safety
///
/// Called only by the component runtime, with the canonical ABI's contract.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn cabi_realloc(
    old: *mut u8,
    old_len: usize,
    align: usize,
    new_len: usize,
) -> *mut u8 {
    use core::alloc::{GlobalAlloc, Layout};
    let ptr = if old_len == 0 {
        if new_len == 0 {
            return align as *mut u8;
        }
        // SAFETY: the runtime passes a power-of-two alignment.
        unsafe { ALLOCATOR.alloc(Layout::from_size_align_unchecked(new_len, align)) }
    } else {
        // SAFETY: `old` was allocated by this function with this layout.
        unsafe {
            ALLOCATOR.realloc(
                old,
                Layout::from_size_align_unchecked(old_len, align),
                new_len,
            )
        }
    };
    if ptr.is_null() {
        core::arch::wasm32::unreachable()
    }
    ptr
}
