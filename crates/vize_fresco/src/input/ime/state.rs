//! IME state management.

use compact_str::CompactString;
use serde::{Deserialize, Serialize};

use super::candidate::CandidateList;
use super::preedit::Preedit;

/// IME state for an input field.
#[derive(Debug, Clone, Default)]
pub struct ImeState {
    /// Whether IME is currently active
    pub active: bool,
    /// Current input mode
    pub mode: ImeMode,
    /// Preedit (uncommitted) text
    pub preedit: Preedit,
    /// Candidate list for selection
    pub candidates: CandidateList,
    /// Whether the IME is composing
    pub composing: bool,
}

impl ImeState {
    /// Create a new IME state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Activate IME.
    pub fn activate(&mut self) {
        self.active = true;
    }

    /// Deactivate IME.
    pub fn deactivate(&mut self) {
        self.active = false;
        self.composing = false;
        self.preedit.clear();
        self.candidates.clear();
    }

    /// Set input mode.
    pub fn set_mode(&mut self, mode: ImeMode) {
        self.mode = mode;
    }

    /// Start composition.
    pub fn start_composition(&mut self) {
        self.composing = true;
    }

    /// End composition.
    pub fn end_composition(&mut self) {
        self.composing = false;
        self.preedit.clear();
        self.candidates.clear();
    }

    /// Update preedit text.
    pub fn update_preedit(&mut self, text: &str, cursor: usize) {
        self.preedit.set_text(text);
        self.preedit.set_cursor(cursor);
    }

    /// Check if currently composing.
    pub fn is_composing(&self) -> bool {
        self.composing && !self.preedit.is_empty()
    }

    /// Check if there are candidates to show.
    pub fn has_candidates(&self) -> bool {
        !self.candidates.is_empty()
    }
}

/// IME input modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ImeMode {
    /// Direct input (no conversion)
    #[default]
    Direct,
    /// Hiragana input
    Hiragana,
    /// Katakana input
    Katakana,
    /// Half-width katakana
    HalfKatakana,
    /// Full-width alphanumeric
    FullWidthAlpha,
    /// Chinese Pinyin input
    Pinyin,
    /// Chinese Wubi input
    Wubi,
    /// Korean Hangul input
    Hangul,
}

impl ImeMode {
    /// Get display name for the mode.
    pub fn display_name(&self) -> &'static str {
        match self {
            ImeMode::Direct => "A",
            ImeMode::Hiragana => "あ",
            ImeMode::Katakana => "ア",
            ImeMode::HalfKatakana => "ｱ",
            ImeMode::FullWidthAlpha => "Ａ",
            ImeMode::Pinyin => "拼",
            ImeMode::Wubi => "五",
            ImeMode::Hangul => "한",
        }
    }

    /// Check if this is a Japanese input mode.
    pub fn is_japanese(&self) -> bool {
        matches!(
            self,
            ImeMode::Hiragana | ImeMode::Katakana | ImeMode::HalfKatakana
        )
    }

    /// Check if this is a Chinese input mode.
    pub fn is_chinese(&self) -> bool {
        matches!(self, ImeMode::Pinyin | ImeMode::Wubi)
    }

    /// Check if this is a Korean input mode.
    pub fn is_korean(&self) -> bool {
        matches!(self, ImeMode::Hangul)
    }
}

/// Events emitted by the IME.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImeEvent {
    /// IME was enabled
    Enabled,
    /// IME was disabled
    Disabled,
    /// Input mode changed
    ModeChanged(ImeMode),
    /// Composition started
    CompositionStart,
    /// Preedit text updated
    PreeditUpdate {
        /// The preedit text
        text: CompactString,
        /// Cursor position within preedit
        cursor: usize,
    },
    /// Text was committed
    Commit(CompactString),
    /// Composition ended
    CompositionEnd,
    /// Candidate list updated
    CandidatesUpdate {
        /// Current candidates
        candidates: Vec<CompactString>,
        /// Selected index
        selected: usize,
    },
}

#[cfg(test)]
mod tests {
    use super::{ImeMode, ImeState};

    #[test]
    fn test_ime_state_new() {
        let state = ImeState::new();
        assert!(!state.active);
        assert!(!state.composing);
        assert_eq!(state.mode, ImeMode::Direct);
    }

    #[test]
    fn test_ime_activate() {
        let mut state = ImeState::new();
        state.activate();
        assert!(state.active);
    }

    #[test]
    fn test_ime_composition() {
        let mut state = ImeState::new();
        state.activate();
        state.start_composition();
        state.update_preedit("にほん", 3);

        assert!(state.is_composing());
        assert_eq!(state.preedit.text(), "にほん");

        state.end_composition();
        assert!(!state.is_composing());
    }

    #[test]
    fn test_ime_mode_display() {
        assert_eq!(ImeMode::Hiragana.display_name(), "あ");
        assert_eq!(ImeMode::Katakana.display_name(), "ア");
        assert_eq!(ImeMode::Direct.display_name(), "A");
    }

    #[test]
    fn test_ime_mode_language() {
        assert!(ImeMode::Hiragana.is_japanese());
        assert!(ImeMode::Pinyin.is_chinese());
        assert!(ImeMode::Hangul.is_korean());
        assert!(!ImeMode::Direct.is_japanese());
    }
}
