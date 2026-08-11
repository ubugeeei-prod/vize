use super::{DiagnosticKeyBinding, DiagnosticKeyChord, DiagnosticWorkspaceKeymap};
use crate::component::DiagnosticWorkspaceCommand;
use crate::input::Key;

impl Default for DiagnosticWorkspaceKeymap {
    /// Build Fresco's portable diagnostic-workspace key contract.
    ///
    /// Arrow keys coexist with Vim-style finding navigation. Filters and modal
    /// actions are press-only; navigation and scrolling may repeat. No default
    /// chord uses Alt, Super, Hyper, or Meta, preserving terminal and window
    /// manager conventions.
    fn default() -> Self {
        use DiagnosticWorkspaceCommand as Command;

        let bindings = vec![
            binding(DiagnosticKeyChord::key(Key::Down), Command::NextFinding),
            binding(DiagnosticKeyChord::char('j'), Command::NextFinding),
            binding(DiagnosticKeyChord::key(Key::Up), Command::PreviousFinding),
            binding(DiagnosticKeyChord::char('k'), Command::PreviousFinding),
            binding(
                DiagnosticKeyChord::key(Key::PageDown),
                Command::PageDownFindings,
            ),
            binding(
                DiagnosticKeyChord::key(Key::PageUp),
                Command::PageUpFindings,
            ),
            binding(DiagnosticKeyChord::key(Key::Home), Command::FirstFinding),
            binding(DiagnosticKeyChord::char('g'), Command::FirstFinding),
            binding(DiagnosticKeyChord::key(Key::End), Command::LastFinding),
            binding(DiagnosticKeyChord::shift('g'), Command::LastFinding),
            binding(DiagnosticKeyChord::char(']'), Command::NextEvidence),
            binding(DiagnosticKeyChord::char('['), Command::PreviousEvidence),
            binding(DiagnosticKeyChord::ctrl('e'), Command::ScrollDetailDown),
            binding(DiagnosticKeyChord::ctrl('y'), Command::ScrollDetailUp),
            binding(DiagnosticKeyChord::ctrl('d'), Command::PageDownDetail),
            binding(DiagnosticKeyChord::ctrl('u'), Command::PageUpDetail),
            binding(DiagnosticKeyChord::key(Key::Tab), Command::FocusNext),
            binding(
                DiagnosticKeyChord::key(Key::BackTab),
                Command::FocusPrevious,
            ),
            binding(DiagnosticKeyChord::char('c'), Command::NextCategory),
            binding(DiagnosticKeyChord::shift('c'), Command::PreviousCategory),
            binding(DiagnosticKeyChord::char('s'), Command::NextSeverity),
            binding(DiagnosticKeyChord::shift('s'), Command::PreviousSeverity),
            binding(DiagnosticKeyChord::char('/'), Command::Search),
            binding(DiagnosticKeyChord::char('o'), Command::OpenSource),
            binding(DiagnosticKeyChord::char('?'), Command::Help),
            binding(DiagnosticKeyChord::key(Key::F(1)), Command::Help),
            binding(DiagnosticKeyChord::char('q'), Command::Exit),
            binding(DiagnosticKeyChord::key(Key::Esc), Command::Exit),
            binding(DiagnosticKeyChord::ctrl('c'), Command::Exit),
        ];

        DiagnosticWorkspaceKeymap::from_valid_defaults(bindings)
    }
}

fn binding(chord: DiagnosticKeyChord, command: DiagnosticWorkspaceCommand) -> DiagnosticKeyBinding {
    DiagnosticKeyBinding::new(chord, command)
}
