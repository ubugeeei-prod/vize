//! Windows-specific IME handling.
//!
//! On Windows, console apps can use:
//! - IMM32 (Input Method Manager)
//! - TSF (Text Services Framework)
//!
//! Windows Terminal has better IME support than legacy cmd.exe.

// Current support: none beyond the generic `TerminalIme`. Fresco does not use
// IMM32 (`ImmGetContext`, `ImmSetCompositionWindow`,
// `ImmGetCompositionString`) or TSF, so it cannot read the preedit string or
// position the composition window. Composition happens in the console host
// (Windows Terminal or conhost); Fresco sees the committed text as key or
// bracketed-paste events.
