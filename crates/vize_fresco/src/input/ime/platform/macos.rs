//! macOS-specific IME handling.
//!
//! On macOS, terminal apps can use the Text Input Source Services API
//! to query and control the system IME. However, direct integration
//! is limited compared to GUI applications.

// Current support: none beyond the generic `TerminalIme`. Fresco does not call
// the Text Input Source Services API (`TISCopyCurrentKeyboardInputSource`),
// so it cannot detect or switch the active input source, and a terminal
// process has no `NSTextInputClient` to integrate with. Composition happens
// in the host terminal; Fresco sees the committed text as key or
// bracketed-paste events.
