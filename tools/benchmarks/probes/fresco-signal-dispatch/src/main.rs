//! Supplemental public-API signal dispatch instruction probe.
//! No terminal backend is active: this isolates saved-action publication and
//! handler/chain dispatch while executing Fresco's actual public hook.

#[cfg(not(all(target_os = "linux", target_arch = "x86_64")))]
compile_error!("this source-only probe targets the existing Linux x86_64 Callgrind lane");

mod instruction;

use std::{
    mem::MaybeUninit,
    ptr,
    sync::atomic::{AtomicI32, AtomicUsize, Ordering},
};
use vize_fresco::{TerminalSignalHookInstallation, install_terminal_signal_hook};

#[global_allocator]
static ALLOCATOR: mimalloc::MiMalloc = mimalloc::MiMalloc;

const FIXTURE: &str = "fixture:fresco-signal-dispatch-authored-v1";
const ENTRY_ERRNO: libc::c_int = libc::E2BIG;
const SIMPLE_EXIT_ERRNO: libc::c_int = libc::EDOM;
const SIGINFO_EXIT_ERRNO: libc::c_int = libc::ERANGE;
static SIMPLE_CALLS: AtomicUsize = AtomicUsize::new(0);
static SIMPLE_SIGNAL: AtomicI32 = AtomicI32::new(0);
static SIMPLE_ERRNO: AtomicI32 = AtomicI32::new(0);
static SIGINFO_CALLS: AtomicUsize = AtomicUsize::new(0);
static SIGINFO_SIGNAL: AtomicI32 = AtomicI32::new(0);
static SIGINFO_ERRNO: AtomicI32 = AtomicI32::new(0);
static SIGINFO_HAS_INFORMATION: AtomicUsize = AtomicUsize::new(0);

extern "C" fn simple_handler(signal: libc::c_int) {
    SIMPLE_CALLS.fetch_add(1, Ordering::Relaxed);
    SIMPLE_SIGNAL.store(signal, Ordering::Relaxed);
    SIMPLE_ERRNO.store(errno::errno().0, Ordering::Relaxed);
    // Fresco must preserve the foreign handler's own errno discipline too.
    errno::set_errno(errno::Errno(SIMPLE_EXIT_ERRNO));
}

extern "C" fn siginfo_handler(
    signal: libc::c_int,
    information: *mut libc::siginfo_t,
    _context: *mut libc::c_void,
) {
    SIGINFO_CALLS.fetch_add(1, Ordering::Relaxed);
    SIGINFO_ERRNO.store(errno::errno().0, Ordering::Relaxed);
    if information.is_null() {
        SIGINFO_SIGNAL.store(signal, Ordering::Relaxed);
    } else {
        SIGINFO_HAS_INFORMATION.store(1, Ordering::Relaxed);
        // SAFETY: the kernel supplies this initialized siginfo_t to the
        // installed SA_SIGINFO handler for this synchronous raise delivery.
        SIGINFO_SIGNAL.store(unsafe { (*information).si_signo }, Ordering::Relaxed);
    }
    errno::set_errno(errno::Errno(SIGINFO_EXIT_ERRNO));
}

fn install_action(signal: libc::c_int, handler: usize, flags: libc::c_int) {
    // SAFETY: the zero representation is valid for libc's Linux sigaction.
    // Required fields and the signal mask are initialized before installation.
    let mut action = unsafe { MaybeUninit::<libc::sigaction>::zeroed().assume_init() };
    action.sa_sigaction = handler;
    action.sa_flags = flags;
    // SAFETY: both calls borrow initialized storage in this local action.
    assert_eq!(unsafe { libc::sigemptyset(&mut action.sa_mask) }, 0);
    assert_eq!(
        unsafe { libc::sigaddset(&mut action.sa_mask, libc::SIGUSR1) },
        0
    );
    // SAFETY: the action matches its selected C ABI and borrows live storage.
    assert_eq!(
        unsafe { libc::sigaction(signal, &action, ptr::null_mut()) },
        0
    );
}

fn assert_installed_mask_and_flags(signal: libc::c_int, expected_flags: libc::c_int) {
    let mut action = MaybeUninit::<libc::sigaction>::zeroed();
    // SAFETY: query-only sigaction initializes the local output on success.
    assert_eq!(
        unsafe { libc::sigaction(signal, ptr::null(), action.as_mut_ptr()) },
        0
    );
    // SAFETY: the successful call initialized action.
    let action = unsafe { action.assume_init() };
    assert_eq!(action.sa_flags & expected_flags, expected_flags);
    assert_ne!(action.sa_flags & libc::SA_SIGINFO, 0);
    // SAFETY: the mask was initialized by sigaction.
    assert_eq!(
        unsafe { libc::sigismember(&action.sa_mask, libc::SIGUSR1) },
        1
    );
}

#[inline(never)]
fn deliver(signal: libc::c_int) -> (libc::c_int, libc::c_int) {
    // Set this inside the window: CString construction and setup may use libc
    // before collection starts. No product call is used as warmup.
    errno::set_errno(errno::Errno(ENTRY_ERRNO));
    // SAFETY: each selected signal has Fresco's wrapper installed by its real
    // public API; the process owns the signal actions for its entire lifetime.
    let status = unsafe { libc::raise(signal) };
    (status, errno::errno().0)
}

fn dispatch_window(id: &str, signal: libc::c_int, expected_exit_errno: libc::c_int) {
    let result = instruction::measure(id, || deliver(std::hint::black_box(signal)));
    instruction::record(
        id,
        FIXTURE,
        "libc-raise-through-real-fresco-handler-no-terminal-v1",
    );
    assert_eq!(result, (0, expected_exit_errno));
}

fn default_disposition_child() -> ! {
    install_action(libc::SIGHUP, libc::SIG_DFL, 0);
    assert!(matches!(
        install_terminal_signal_hook().unwrap(),
        TerminalSignalHookInstallation::Installed
    ));
    eprintln!("FRESCO_SIGNAL_DEFAULT_READY");
    // SAFETY: the public hook wrapped the native SIGHUP default disposition.
    // Its default branch must restore that action and re-raise to terminate.
    unsafe { libc::raise(libc::SIGHUP) };
    std::process::exit(102)
}

fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args == ["--default-sighup"] {
        default_disposition_child();
    }
    assert!(args.is_empty(), "unexpected probe arguments");
    assert!(
        instruction::requested(),
        "run through the supplied Callgrind driver"
    );
    instruction::initialize();

    install_action(
        libc::SIGTERM,
        simple_handler as *const () as usize,
        libc::SA_RESTART,
    );
    install_action(
        libc::SIGINT,
        siginfo_handler as *const () as usize,
        libc::SA_SIGINFO,
    );
    install_action(libc::SIGQUIT, libc::SIG_IGN, 0);
    install_action(libc::SIGHUP, libc::SIG_DFL, 0);

    let installed = instruction::measure("fresco.signal.install", install_terminal_signal_hook);
    instruction::record("fresco.signal.install", FIXTURE, "real-public-install-v1");
    assert!(matches!(
        installed.unwrap(),
        TerminalSignalHookInstallation::Installed
    ));
    assert_installed_mask_and_flags(libc::SIGTERM, libc::SA_RESTART);
    assert_installed_mask_and_flags(libc::SIGINT, libc::SA_SIGINFO);
    assert_installed_mask_and_flags(libc::SIGQUIT, 0);

    dispatch_window(
        "fresco.signal.dispatch.simple",
        libc::SIGTERM,
        SIMPLE_EXIT_ERRNO,
    );
    assert_eq!(SIMPLE_CALLS.load(Ordering::Relaxed), 1);
    assert_eq!(SIGINFO_CALLS.load(Ordering::Relaxed), 0);
    assert_eq!(SIMPLE_SIGNAL.load(Ordering::Relaxed), libc::SIGTERM);
    assert_eq!(SIMPLE_ERRNO.load(Ordering::Relaxed), ENTRY_ERRNO);

    dispatch_window(
        "fresco.signal.dispatch.siginfo",
        libc::SIGINT,
        SIGINFO_EXIT_ERRNO,
    );
    assert_eq!(SIMPLE_CALLS.load(Ordering::Relaxed), 1);
    assert_eq!(SIGINFO_CALLS.load(Ordering::Relaxed), 1);
    assert_eq!(SIGINFO_SIGNAL.load(Ordering::Relaxed), libc::SIGINT);
    assert_eq!(SIGINFO_ERRNO.load(Ordering::Relaxed), ENTRY_ERRNO);
    assert_eq!(SIGINFO_HAS_INFORMATION.load(Ordering::Relaxed), 1);

    dispatch_window("fresco.signal.dispatch.ignore", libc::SIGQUIT, ENTRY_ERRNO);
    assert_eq!(SIMPLE_CALLS.load(Ordering::Relaxed), 1);
    assert_eq!(SIGINFO_CALLS.load(Ordering::Relaxed), 1);

    println!(
        "{}",
        serde_json::json!({
            "oracle": "fresco-public-signal-dispatch-v1",
            "simple_calls": SIMPLE_CALLS.load(Ordering::Relaxed),
            "simple_signal": SIMPLE_SIGNAL.load(Ordering::Relaxed),
            "simple_entry_errno": SIMPLE_ERRNO.load(Ordering::Relaxed),
            "simple_exit_errno": SIMPLE_EXIT_ERRNO,
            "siginfo_calls": SIGINFO_CALLS.load(Ordering::Relaxed),
            "siginfo_signal": SIGINFO_SIGNAL.load(Ordering::Relaxed),
            "siginfo_entry_errno": SIGINFO_ERRNO.load(Ordering::Relaxed),
            "siginfo_exit_errno": SIGINFO_EXIT_ERRNO,
            "siginfo_has_information": SIGINFO_HAS_INFORMATION.load(Ordering::Relaxed),
            "ignored_exit_errno": ENTRY_ERRNO,
            "mask_contains_sigusr1": true,
            "terminal_backend_active": false,
        })
    );
}
