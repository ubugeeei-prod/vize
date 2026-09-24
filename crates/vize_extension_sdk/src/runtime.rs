//! The runtime an import-free `no_std` guest provides itself (`runtime`
//! feature, `wasm32` only).
//!
//! On `wasm32-wasip2` these normally come from the C library, but the
//! `input-dialect` world imports no host function, so a guest links none.

use core::alloc::{GlobalAlloc, Layout};

#[global_allocator]
static ALLOCATOR: dlmalloc::GlobalDlmalloc = dlmalloc::GlobalDlmalloc;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    core::arch::wasm32::unreachable()
}

/// The canonical ABI's allocation entry point.
///
/// # Safety
///
/// Called only by the component runtime, under the canonical ABI's contract.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn cabi_realloc(
    old: *mut u8,
    old_len: usize,
    align: usize,
    new_len: usize,
) -> *mut u8 {
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

/// `memcmp`: `wasm32` has no compare instruction and no C library links in.
/// Volatile reads keep the optimizer from turning the loop into a call to
/// itself.
///
/// # Safety
///
/// `a` and `b` are valid for `len` bytes (the C contract).
#[expect(
    suspicious_runtime_symbol_definitions,
    reason = "no C library links into the guest, so this is its only `memcmp`, not a clash with one"
)]
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
#[expect(
    suspicious_runtime_symbol_definitions,
    reason = "no C library links into the guest, so this is its only `bcmp`"
)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bcmp(a: *const u8, b: *const u8, len: usize) -> i32 {
    // SAFETY: forwarded under the same contract.
    unsafe { memcmp(a, b, len) }
}
