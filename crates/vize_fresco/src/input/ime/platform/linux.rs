//! Linux-specific IME handling.
//!
//! On Linux, terminal apps typically interact with IME through:
//! - ibus (IBus)
//! - fcitx (Fcitx5)
//! - DBus interfaces
//!
//! Direct integration is challenging in terminal contexts.

// Current support: none beyond the generic `TerminalIme`. Fresco does not
// talk to IBus or Fcitx5 over DBus and does not speak XIM. Composition
// happens in the host terminal's own IME; Fresco sees the committed text as
// key or bracketed-paste events. It cannot query or switch the active input
// method and cannot position a preedit window.
