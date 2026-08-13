//! Fresco - Vue TUI Framework
//!
//! A high-performance Terminal User Interface framework for Vue.js,
//! similar to React Ink but built with Rust for performance.
//!
//! ## Stability
//!
//! **Incubating.** The API or crate may be replaced or removed in any release. See the
//! [Rust crate support tiers](https://github.com/ubugeeei-prod/vize/blob/main/docs/content/stability.md#rust-crate-support-tiers).
//!
//! # Features
//!
//! - **Terminal Control**: Cross-platform terminal handling via crossterm
//! - **Flexbox Layout**: Layout engine powered by taffy
//! - **CJK Support**: Full Unicode text handling including Japanese IME
//! - **Efficient Rendering**: Double-buffered differential rendering
//! - **Diagnostic Workspaces**: Stable, virtualized master-detail navigation
//! - **Headless Assertions**: Deterministic visual and semantic frame snapshots
//!
//! # Diagnostic Workspace Contracts
//!
//! Fresco's diagnostic workspace primitives are semantic state contracts, not a
//! reporter-specific terminal runtime. Diagnostic tools own their immutable
//! report model and pass stable item keys into [`DiagnosticWorkspaceState`],
//! which preserves selection, focus, scrolling, and related-evidence navigation
//! across filtering, reordering, and resize.
//!
//! Defaults are intentionally conservative and deterministic:
//!
//! - [`DiagnosticWorkspaceOptions::default`] switches to split master-detail
//!   layout at 80 columns, assigns 40% of split width to the finding list,
//!   reserves 3 chrome rows, and retains 2 virtualized overscan rows on each
//!   side of a viewport.
//! - [`TerminalProfileOptions::default`] selects automatic color, Unicode, and
//!   interactivity resolution and a 60-cell narrow-layout threshold;
//!   [`TerminalCapabilities::resolve`] consumes those preferences together with
//!   explicit probe data to mark widths below the threshold as narrow.
//! - [`terminal::TerminalOptions::default`] acquires raw mode, alternate
//!   screen, bracketed paste, and hidden cursor, while leaving mouse capture
//!   disabled.
//!
//! Unsupported or downgraded terminal capabilities are represented by
//! [`CapabilityDecision`] and [`CapabilityReason`] instead of being inferred
//! from rendered text. `NO_COLOR`, forced color, non-UTF-8 locales, redirected
//! output, `TERM=dumb`, CI, and Fresco-specific Unicode or interactivity
//! overrides all produce stable reasons suitable for snapshots and docs.
//!
//! Focus is semantic and renderer-independent. Findings and detail remain
//! focusable even in zero-row viewports; evidence focus is available only when a
//! related-evidence item is selected. Narrow terminals use stacked panes
//! selected from semantic focus, while split terminals present findings and
//! detail simultaneously.
//!
//! Terminal restoration is explicit and best-effort. [`Backend::restore`]
//! attempts every mode owned or possibly owned by the backend in deterministic
//! order, including raw mode, cursor visibility and shape, bracketed paste,
//! mouse capture, and alternate screen. Failed cleanups remain owned for a later
//! retry or [`Drop`], and [`TerminalRestorationError`] reports every failure.
//! Process panic and signal supervision are exposed separately through
//! [`install_terminal_panic_hook`] and [`install_terminal_signal_hook`].
//!
//! Performance limits are part of the API shape. Virtualized lists materialize
//! only the viewport plus configured overscan, workspace state retains constant
//! selection and viewport data instead of report rows, and [`FrameTelemetry`]
//! exposes layout time, paint time, output time, changed cells, bytes written,
//! retained nodes, and dropped or coalesced frame requests.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │                    Vue Components                        │
//! │                  (Box, Text, Input)                       │
//! └─────────────────────────────────────────────────────────┘
//!                           │
//!                           ▼
//! ┌─────────────────────────────────────────────────────────┐
//! │                  Vue Custom Renderer                     │
//! │                    (TypeScript)                          │
//! └─────────────────────────────────────────────────────────┘
//!                           │
//!                           ▼
//! ┌─────────────────────────────────────────────────────────┐
//! │                   NAPI Bindings                          │
//! │             (Rust <-> Node.js bridge)                    │
//! └─────────────────────────────────────────────────────────┘
//!                           │
//!         ┌─────────────────┼─────────────────┐
//!         ▼                 ▼                 ▼
//! ┌───────────────┐ ┌───────────────┐ ┌───────────────┐
//! │   Terminal    │ │    Layout     │ │    Render     │
//! │   (backend,   │ │   (taffy,     │ │   (tree,      │
//! │    buffer)    │ │    flex)      │ │    diff)      │
//! └───────────────┘ └───────────────┘ └───────────────┘
//!         │                                   │
//!         ▼                                   ▼
//! ┌───────────────┐                 ┌───────────────┐
//! │     Input     │                 │     Text      │
//! │  (keyboard,   │                 │   (width,     │
//! │   mouse, ime) │                 │    segment)   │
//! └───────────────┘                 └───────────────┘
//! ```

pub mod component;
pub mod headless;
pub mod input;
pub mod layout;
pub mod render;
pub mod terminal;
pub mod text;

#[cfg(feature = "napi")]
pub mod napi;

// Re-exports for convenience
pub use component::{
    BoxNode, DiagnosticKeyBinding, DiagnosticKeyChord, DiagnosticKeymapError,
    DiagnosticPresentation, DiagnosticPresentationError, DiagnosticPresentationKind,
    DiagnosticPresentationProfile, DiagnosticTone, DiagnosticWorkspaceAction,
    DiagnosticWorkspaceCommand, DiagnosticWorkspaceCommandOutcome, DiagnosticWorkspaceFocus,
    DiagnosticWorkspaceKeymap, DiagnosticWorkspaceLayout, DiagnosticWorkspaceMode,
    DiagnosticWorkspaceOptions, DiagnosticWorkspacePane, DiagnosticWorkspaceState, InputNode,
    TextNode, VirtualListNavigation, VirtualListState, VirtualWindow,
};
pub use headless::{
    AnnouncementPoliteness, HeadlessAnnouncement, HeadlessPresentation, HeadlessRenderError,
    HeadlessRenderer, HeadlessSemanticNode, HeadlessSnapshot, SemanticRole, SemanticState,
};
pub use input::{Event, ImeState, Key, KeyEvent, KeyEventKind, KeyModifiers, MouseEvent};
pub use layout::{FlexStyle, LayoutEngine, Rect};
pub use render::{
    FRAME_TELEMETRY_SCHEMA_VERSION, FrameActivityTelemetry, FrameCoalescer, FrameRenderError,
    FrameRenderer, FrameRequestOutcome, FrameTelemetry, RenderNode, RenderTree,
};
pub use terminal::{
    Backend, Buffer, CapabilityDecision, CapabilityReason, Cell, ColorPreference, ColorSupport,
    Cursor, FeaturePreference, FrameOutputTelemetry, TerminalCapabilities, TerminalCapabilityProbe,
    TerminalCleanupFailure, TerminalMode, TerminalPanicHookError, TerminalPanicHookInstallation,
    TerminalProfileOptions, TerminalRestorationError, TerminalSessionAcquireError,
    TerminalSessionPhase, TerminalSessionState, TerminalSignalHookError,
    TerminalSignalHookInstallation, TerminalSignalRollbackFailure, install_terminal_panic_hook,
    install_terminal_signal_hook,
};
pub use text::{TextSegment, TextWidth, TextWrap};

/// Fresco version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
