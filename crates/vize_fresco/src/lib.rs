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
    TerminalSessionPhase, TerminalSessionState, install_terminal_panic_hook,
};
pub use text::{TextSegment, TextWidth, TextWrap};

/// Fresco version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
