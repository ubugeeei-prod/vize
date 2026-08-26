//! Deterministic physical-key to semantic-command mapping.

mod defaults;

#[cfg(test)]
mod tests;

use std::fmt;

use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use vize_s0::{String, ToCompactString};

use super::DiagnosticWorkspaceCommand;
use crate::input::{Key, KeyEvent, KeyEventKind, KeyModifiers};

/// A canonical physical key and modifier combination.
///
/// Canonicalization accounts for common terminal differences: uppercase ASCII
/// letters imply Shift, Shift is ignored when a character is itself a shifted
/// ASCII symbol such as `?`, and `BackTab` already carries its Shift meaning.
/// All non-Shift modifiers remain exact. Consequently, `Ctrl+C` never matches
/// plain `C`, and an unsupported modified chord fails closed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DiagnosticKeyChord {
    /// Physical or character key.
    pub key: Key,
    /// Exact canonical modifier set.
    pub modifiers: KeyModifiers,
}

impl DiagnosticKeyChord {
    /// Create and canonicalize a chord.
    pub fn new(key: Key, modifiers: KeyModifiers) -> Self {
        Self { key, modifiers }.canonicalized()
    }

    /// Create an unmodified chord.
    pub fn key(key: Key) -> Self {
        Self::new(key, KeyModifiers::NONE)
    }

    /// Create an unmodified character chord.
    pub fn char(character: char) -> Self {
        Self::key(Key::Char(character))
    }

    /// Create a Ctrl-modified character chord.
    pub fn ctrl(character: char) -> Self {
        Self::new(
            Key::Char(character),
            KeyModifiers {
                ctrl: true,
                ..KeyModifiers::NONE
            },
        )
    }

    /// Create a Shift-modified character chord.
    pub fn shift(character: char) -> Self {
        Self::new(
            Key::Char(character),
            KeyModifiers {
                shift: true,
                ..KeyModifiers::NONE
            },
        )
    }

    /// Return a deterministic, compact label for help and key hints.
    pub fn label(self) -> String {
        self.to_compact_string()
    }

    fn from_event(event: &KeyEvent) -> Self {
        Self::new(event.key, event.modifiers)
    }

    fn canonicalized(mut self) -> Self {
        match self.key {
            Key::Char(character) if character.is_ascii_uppercase() => {
                self.key = Key::Char(character.to_ascii_lowercase());
                self.modifiers.shift = true;
            }
            Key::Char(character) if is_shifted_ascii_symbol(character) => {
                self.modifiers.shift = false;
            }
            Key::BackTab => self.modifiers.shift = false,
            _ => {}
        }
        self
    }
}

fn is_shifted_ascii_symbol(character: char) -> bool {
    matches!(
        character,
        '~' | '!'
            | '@'
            | '#'
            | '$'
            | '%'
            | '^'
            | '&'
            | '*'
            | '('
            | ')'
            | '_'
            | '+'
            | '{'
            | '}'
            | '|'
            | ':'
            | '"'
            | '<'
            | '>'
            | '?'
    )
}

impl fmt::Display for DiagnosticKeyChord {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let chord = self.canonicalized();
        for (active, name) in [
            (chord.modifiers.ctrl, "Ctrl"),
            (chord.modifiers.alt, "Alt"),
            (chord.modifiers.shift, "Shift"),
            (chord.modifiers.super_key, "Super"),
            (chord.modifiers.hyper, "Hyper"),
            (chord.modifiers.meta, "Meta"),
        ] {
            if active {
                write!(formatter, "{name}+")?;
            }
        }
        write_key_label(formatter, chord.key)
    }
}

/// One deterministic key-to-command assignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiagnosticKeyBinding {
    /// Canonical key chord.
    pub chord: DiagnosticKeyChord,
    /// Semantic command emitted by the chord.
    pub command: DiagnosticWorkspaceCommand,
}

impl DiagnosticKeyBinding {
    /// Create a binding and canonicalize its chord.
    pub fn new(chord: DiagnosticKeyChord, command: DiagnosticWorkspaceCommand) -> Self {
        Self {
            chord: chord.canonicalized(),
            command,
        }
    }
}

/// Invalid custom diagnostic keymap.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DiagnosticKeymapError {
    /// Two bindings canonicalize to the same physical chord.
    #[error("diagnostic key chord {chord} maps to both {existing:?} and {duplicate:?}")]
    DuplicateChord {
        /// Conflicting canonical chord.
        chord: DiagnosticKeyChord,
        /// Command registered first.
        existing: DiagnosticWorkspaceCommand,
        /// Later conflicting command.
        duplicate: DiagnosticWorkspaceCommand,
    },
}

/// Validated diagnostic-workspace keyboard contract.
///
/// Construction allocates once. Event resolution performs one canonicalization
/// and one hash lookup with no allocation. Binding order is retained for stable
/// help presentation, while lookup behavior is independent of that order.
///
/// ```
/// # #[cfg(not(feature = "napi"))]
/// # {
/// use vize_fresco::{DiagnosticWorkspaceCommand, DiagnosticWorkspaceKeymap, KeyEvent};
///
/// let keymap = DiagnosticWorkspaceKeymap::default();
/// assert_eq!(
///     keymap.resolve(&KeyEvent::char('/')),
///     Some(DiagnosticWorkspaceCommand::Search),
/// );
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct DiagnosticWorkspaceKeymap {
    bindings: Vec<DiagnosticKeyBinding>,
    commands: FxHashMap<DiagnosticKeyChord, DiagnosticWorkspaceCommand>,
}

impl DiagnosticWorkspaceKeymap {
    /// Validate and build a custom keymap.
    ///
    /// Duplicate chords fail closed, including collisions introduced by
    /// terminal canonicalization such as uppercase `C` and `Shift+C`.
    pub fn new(
        bindings: impl IntoIterator<Item = DiagnosticKeyBinding>,
    ) -> Result<Self, DiagnosticKeymapError> {
        let bindings = bindings
            .into_iter()
            .map(|binding| DiagnosticKeyBinding::new(binding.chord, binding.command))
            .collect::<Vec<_>>();
        let mut commands = FxHashMap::with_capacity_and_hasher(bindings.len(), Default::default());
        for binding in &bindings {
            let chord = binding.chord.canonicalized();
            if let Some(existing) = commands.insert(chord, binding.command) {
                return Err(DiagnosticKeymapError::DuplicateChord {
                    chord,
                    existing,
                    duplicate: binding.command,
                });
            }
        }
        Ok(Self { bindings, commands })
    }

    /// Return the stable binding order used by help presentations.
    pub fn bindings(&self) -> &[DiagnosticKeyBinding] {
        &self.bindings
    }

    /// Resolve a normalized terminal event into a semantic command.
    ///
    /// Release events are ignored. Repeat events resolve only for commands
    /// whose repeat behavior is explicitly safe. Unknown and over-modified
    /// events return `None`.
    pub fn resolve(&self, event: &KeyEvent) -> Option<DiagnosticWorkspaceCommand> {
        if event.kind == KeyEventKind::Release {
            return None;
        }
        let command = self
            .commands
            .get(&DiagnosticKeyChord::from_event(event))
            .copied()?;
        if event.kind == KeyEventKind::Repeat && !command.accepts_repeat() {
            return None;
        }
        Some(command)
    }

    pub(super) fn from_valid_defaults(bindings: Vec<DiagnosticKeyBinding>) -> Self {
        let commands = bindings
            .iter()
            .map(|binding| (binding.chord.canonicalized(), binding.command))
            .collect();
        Self { bindings, commands }
    }
}

fn write_key_label(formatter: &mut fmt::Formatter<'_>, key: Key) -> fmt::Result {
    match key {
        Key::Char(character) => write!(formatter, "{character}"),
        Key::F(number) => write!(formatter, "F{number}"),
        Key::Backspace => formatter.write_str("Backspace"),
        Key::Enter => formatter.write_str("Enter"),
        Key::Left => formatter.write_str("Left"),
        Key::Right => formatter.write_str("Right"),
        Key::Up => formatter.write_str("Up"),
        Key::Down => formatter.write_str("Down"),
        Key::Home => formatter.write_str("Home"),
        Key::End => formatter.write_str("End"),
        Key::PageUp => formatter.write_str("PgUp"),
        Key::PageDown => formatter.write_str("PgDn"),
        Key::Tab => formatter.write_str("Tab"),
        Key::BackTab => formatter.write_str("Shift+Tab"),
        Key::Delete => formatter.write_str("Delete"),
        Key::Insert => formatter.write_str("Insert"),
        Key::Esc => formatter.write_str("Esc"),
        Key::CapsLock => formatter.write_str("CapsLock"),
        Key::ScrollLock => formatter.write_str("ScrollLock"),
        Key::NumLock => formatter.write_str("NumLock"),
        Key::PrintScreen => formatter.write_str("PrintScreen"),
        Key::Pause => formatter.write_str("Pause"),
        Key::Menu => formatter.write_str("Menu"),
        Key::Null => formatter.write_str("Null"),
    }
}
