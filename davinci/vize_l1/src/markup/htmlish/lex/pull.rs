//! Pull one tag at a caller-chosen `<`.
//!
//! A formatter that walks text itself still needs the lexer's tag grammar.
//! [`Lexer::pull_tag`] runs the shared state machine from the `<` at `start`
//! through the tag's own `>` (or EOF) and stops: text, comment, raw-text and
//! interpolation states are never entered, so the caller keeps owning them.
//! Every event is the one [`Lexer::run`] would push for the same tag.

use super::{Lexer, State, is_tag_start_char};
use crate::markup::profile::Profile;
use crate::markup::token::{LexErrorCode, Sink};

/// How a pulled tag ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PulledTag {
    /// The byte after `<` starts no tag, comment-like or declaration
    /// framing that this pull owns; the `<` is text.
    NotTag,
    /// An opening tag. `end` is the byte after its `>`, or the input length
    /// when EOF ended it (`terminated` is then false).
    Open { end: usize, terminated: bool },
    /// A closing tag, with the same `end`/`terminated` contract.
    Close { end: usize, terminated: bool },
}

impl<'a, P: Profile, S: Sink> Lexer<'a, P, S> {
    /// The sink receiving this lexer's events.
    #[inline]
    pub fn sink_mut(&mut self) -> &mut S {
        &mut self.sink
    }

    /// Lex the single tag whose `<` is at `start`, pushing its events into
    /// the sink. `<!`, `<?` and every other non-tag byte report
    /// [`PulledTag::NotTag`] without events.
    #[inline]
    pub fn pull_tag(&mut self, start: usize) -> PulledTag {
        let input = self.input;
        self.section_start = start;
        self.after_quoted_attr_value = false;
        self.in_rcdata = false;
        self.current_sequence = None;
        self.sequence_index = 0;
        let closing = input.get(start + 1) == Some(&b'/');
        self.index = start + 1;
        self.state = State::BeforeTagName;
        while let Some(&c) = input.get(self.index) {
            match self.state {
                State::BeforeTagName => {
                    if c == b'/' {
                        self.state = State::BeforeClosingTagName;
                    } else if is_tag_start_char(c) {
                        self.state_before_tag_name(c);
                    } else {
                        return PulledTag::NotTag;
                    }
                }
                State::InTagName => self.state_in_tag_name(c),
                State::InSelfClosingTag => self.state_in_self_closing_tag(c),
                State::BeforeClosingTagName => self.state_before_closing_tag_name(c),
                State::InClosingTagName => self.state_in_closing_tag_name(c),
                State::AfterClosingTagName => self.state_after_closing_tag_name(c),
                State::BeforeAttrName => self.state_before_attr_name(c),
                State::InAttrName => self.state_in_attr_name(c),
                State::InDirName => self.state_in_dir_name(c),
                State::InDirArg => self.state_in_dir_arg(c),
                State::InDirDynamicArg => self.state_in_dir_dynamic_arg(c),
                State::InDirModifier => self.state_in_dir_modifier(c),
                State::AfterAttrName => self.state_after_attr_name(c),
                State::BeforeAttrValue => self.state_before_attr_value(c),
                State::InAttrValueDq => self.state_in_attr_value_dq(c),
                State::InAttrValueSq => self.state_in_attr_value_sq(c),
                State::InAttrValueNq => self.state_in_attr_value_nq(c),
                State::InEntity => self.state_in_entity(),
                State::BeforeSpecialS => self.state_before_special_s(c),
                State::BeforeSpecialT => self.state_before_special_t(c),
                State::SpecialStartSequence => self.state_special_start_sequence(c),
                State::InTagComment => self.state_in_tag_comment(c),
                _ => return PulledTag::NotTag,
            }
            self.index += 1;
            if matches!(self.state, State::Text | State::InRCDATA) {
                let end = self.index;
                return if closing {
                    PulledTag::Close {
                        end,
                        terminated: true,
                    }
                } else {
                    PulledTag::Open {
                        end,
                        terminated: true,
                    }
                };
            }
        }
        self.pull_tag_at_eof(closing)
    }

    #[cold]
    #[inline(never)]
    fn pull_tag_at_eof(&mut self, closing: bool) -> PulledTag {
        if self.state == State::BeforeTagName {
            return PulledTag::NotTag;
        }
        self.sink.on_error(LexErrorCode::EofInTag, self.index);
        self.recover_incomplete_tag_at_eof();
        let end = self.input.len();
        if closing {
            PulledTag::Close {
                end,
                terminated: false,
            }
        } else {
            PulledTag::Open {
                end,
                terminated: false,
            }
        }
    }
}
