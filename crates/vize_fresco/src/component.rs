//! Primitive component nodes.
//!
//! Provides the core building blocks for TUI:
//! - BoxNode: Container with flexbox layout
//! - TextNode: Text display
//! - InputNode: Text input with IME support

mod box_node;
mod diagnostic_workspace;
mod input_node;
mod text_node;
mod virtual_list;

#[cfg(test)]
mod virtual_list_tests;

pub use box_node::BoxNode;
pub use diagnostic_workspace::{
    DiagnosticKeyBinding, DiagnosticKeyChord, DiagnosticKeymapError, DiagnosticPresentation,
    DiagnosticPresentationError, DiagnosticPresentationKind, DiagnosticPresentationProfile,
    DiagnosticTone, DiagnosticWorkspaceAction, DiagnosticWorkspaceCommand,
    DiagnosticWorkspaceCommandOutcome, DiagnosticWorkspaceFocus, DiagnosticWorkspaceKeymap,
    DiagnosticWorkspaceLayout, DiagnosticWorkspaceMode, DiagnosticWorkspaceOptions,
    DiagnosticWorkspacePane, DiagnosticWorkspaceState,
};
pub use input_node::InputNode;
pub use text_node::TextNode;
pub use virtual_list::{VirtualListNavigation, VirtualListState, VirtualWindow};
