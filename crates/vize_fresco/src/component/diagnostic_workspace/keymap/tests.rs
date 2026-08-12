use super::{
    DiagnosticKeyBinding, DiagnosticKeyChord, DiagnosticKeymapError, DiagnosticWorkspaceKeymap,
};
use crate::component::DiagnosticWorkspaceCommand as Command;
use crate::input::{Key, KeyEvent, KeyEventKind, KeyModifiers};

#[test]
fn defaults_cover_every_required_diagnostic_interaction() {
    let keymap = DiagnosticWorkspaceKeymap::default();
    let commands = keymap
        .bindings()
        .iter()
        .map(|binding| binding.command)
        .collect::<Vec<_>>();

    for required in [
        Command::NextFinding,
        Command::PreviousFinding,
        Command::NextCategory,
        Command::PreviousCategory,
        Command::NextSeverity,
        Command::PreviousSeverity,
        Command::Search,
        Command::NextEvidence,
        Command::PreviousEvidence,
        Command::OpenSource,
        Command::Help,
        Command::Exit,
    ] {
        assert!(commands.contains(&required), "missing {required:?}");
    }

    DiagnosticWorkspaceKeymap::new(keymap.bindings().iter().copied()).unwrap();
}

#[test]
fn default_navigation_supports_arrows_vim_keys_and_safe_repeat() {
    let keymap = DiagnosticWorkspaceKeymap::default();

    assert_eq!(
        keymap.resolve(&KeyEvent::key(Key::Down)),
        Some(Command::NextFinding)
    );
    assert_eq!(
        keymap.resolve(&KeyEvent::char('k')),
        Some(Command::PreviousFinding)
    );
    assert_eq!(
        keymap.resolve(&event(
            Key::Char('j'),
            KeyModifiers::NONE,
            KeyEventKind::Repeat
        )),
        Some(Command::NextFinding)
    );
    assert_eq!(
        keymap.resolve(&event(
            Key::Char('j'),
            KeyModifiers::NONE,
            KeyEventKind::Release
        )),
        None
    );
}

#[test]
fn actions_are_press_only_and_unknown_modifiers_fail_closed() {
    let keymap = DiagnosticWorkspaceKeymap::default();

    assert_eq!(
        keymap.resolve(&event(
            Key::Char('c'),
            KeyModifiers::NONE,
            KeyEventKind::Repeat
        )),
        None
    );
    assert_eq!(
        keymap.resolve(&event(
            Key::Char('q'),
            KeyModifiers {
                alt: true,
                ..KeyModifiers::NONE
            },
            KeyEventKind::Press,
        )),
        None
    );
    assert_eq!(
        keymap.resolve(&event(
            Key::Char('/'),
            KeyModifiers {
                shift: true,
                ..KeyModifiers::NONE
            },
            KeyEventKind::Press,
        )),
        None
    );
    assert_eq!(
        keymap.resolve(&event(
            Key::Char('c'),
            KeyModifiers {
                ctrl: true,
                shift: true,
                ..KeyModifiers::NONE
            },
            KeyEventKind::Press,
        )),
        None
    );
    assert_eq!(
        keymap.resolve(&event(
            Key::Char('c'),
            KeyModifiers {
                ctrl: true,
                ..KeyModifiers::NONE
            },
            KeyEventKind::Press,
        )),
        Some(Command::Exit)
    );
}

#[test]
fn terminal_shift_variants_resolve_to_one_canonical_chord() {
    let keymap = DiagnosticWorkspaceKeymap::default();

    assert_eq!(
        keymap.resolve(&KeyEvent::char('C')),
        Some(Command::PreviousCategory)
    );
    assert_eq!(
        keymap.resolve(&event(
            Key::Char('c'),
            KeyModifiers {
                shift: true,
                ..KeyModifiers::NONE
            },
            KeyEventKind::Press,
        )),
        Some(Command::PreviousCategory)
    );
    assert_eq!(
        keymap.resolve(&event(
            Key::Char('?'),
            KeyModifiers {
                shift: true,
                ..KeyModifiers::NONE
            },
            KeyEventKind::Press,
        )),
        Some(Command::Help)
    );
    assert_eq!(
        keymap.resolve(&event(
            Key::BackTab,
            KeyModifiers {
                shift: true,
                ..KeyModifiers::NONE
            },
            KeyEventKind::Press,
        )),
        Some(Command::FocusPrevious)
    );
}

#[test]
fn custom_keymaps_reject_collisions_after_canonicalization() {
    let result = DiagnosticWorkspaceKeymap::new([
        DiagnosticKeyBinding::new(DiagnosticKeyChord::char('C'), Command::NextCategory),
        DiagnosticKeyBinding::new(DiagnosticKeyChord::shift('c'), Command::PreviousCategory),
    ]);

    assert_eq!(
        result.unwrap_err(),
        DiagnosticKeymapError::DuplicateChord {
            chord: DiagnosticKeyChord::shift('c'),
            existing: Command::NextCategory,
            duplicate: Command::PreviousCategory,
        }
    );
}

#[test]
fn custom_keymaps_preserve_help_order_and_resolve_exactly() {
    let bindings = [
        DiagnosticKeyBinding::new(DiagnosticKeyChord::char('x'), Command::Help),
        DiagnosticKeyBinding::new(DiagnosticKeyChord::key(Key::F(12)), Command::Exit),
    ];
    let keymap = DiagnosticWorkspaceKeymap::new(bindings).unwrap();

    assert_eq!(keymap.bindings(), &bindings);
    assert_eq!(keymap.resolve(&KeyEvent::char('x')), Some(Command::Help));
    assert_eq!(
        keymap.resolve(&KeyEvent::key(Key::F(12))),
        Some(Command::Exit)
    );
    assert_eq!(keymap.resolve(&KeyEvent::char('q')), None);
}

#[test]
fn custom_keymaps_canonicalize_public_field_construction_for_help_and_wire_output() {
    let raw = DiagnosticKeyBinding {
        chord: DiagnosticKeyChord {
            key: Key::Char('C'),
            modifiers: KeyModifiers::NONE,
        },
        command: Command::PreviousCategory,
    };
    let keymap = DiagnosticWorkspaceKeymap::new([raw]).unwrap();

    assert_eq!(keymap.bindings()[0].chord, DiagnosticKeyChord::shift('c'));
    assert_eq!(keymap.bindings()[0].chord.label(), "Shift+c");
}

#[test]
fn labels_are_compact_deterministic_and_semantically_shifted() {
    assert_eq!(DiagnosticKeyChord::ctrl('c').label(), "Ctrl+c");
    assert_eq!(DiagnosticKeyChord::char('C').label(), "Shift+c");
    assert_eq!(DiagnosticKeyChord::char('?').label(), "?");
    assert_eq!(DiagnosticKeyChord::key(Key::PageDown).label(), "PgDn");
    assert_eq!(DiagnosticKeyChord::key(Key::BackTab).label(), "Shift+Tab");
}

#[test]
fn bindings_round_trip_without_changing_command_wire_names() {
    let binding = DiagnosticKeyBinding::new(DiagnosticKeyChord::ctrl('d'), Command::PageDownDetail);
    let json = serde_json::to_string(&binding).unwrap();
    assert!(json.contains("page-down-detail"));
    assert_eq!(
        serde_json::from_str::<DiagnosticKeyBinding>(&json).unwrap(),
        binding
    );
}

fn event(key: Key, modifiers: KeyModifiers, kind: KeyEventKind) -> KeyEvent {
    KeyEvent {
        key,
        modifiers,
        kind,
    }
}
