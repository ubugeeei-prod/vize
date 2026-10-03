//! Downstream witness for the public tokenizer paths shipped in v0.429.1.
//! Keep this crate independent from the Vize workspace and from `vize_l1`.

use std::cell::Cell;
use std::rc::Rc;
use vize_armature as armature;
use vize_armature::tokenizer;

const STATES: [armature::State; 35] = [
    tokenizer::State::Text,
    tokenizer::State::InterpolationOpen,
    tokenizer::State::Interpolation,
    tokenizer::State::InterpolationClose,
    tokenizer::State::BeforeTagName,
    tokenizer::State::InTagName,
    tokenizer::State::InSelfClosingTag,
    tokenizer::State::BeforeClosingTagName,
    tokenizer::State::InClosingTagName,
    tokenizer::State::AfterClosingTagName,
    tokenizer::State::BeforeAttrName,
    tokenizer::State::InTagComment,
    tokenizer::State::InAttrName,
    tokenizer::State::InDirName,
    tokenizer::State::InDirArg,
    tokenizer::State::InDirDynamicArg,
    tokenizer::State::InDirModifier,
    tokenizer::State::AfterAttrName,
    tokenizer::State::BeforeAttrValue,
    tokenizer::State::InAttrValueDq,
    tokenizer::State::InAttrValueSq,
    tokenizer::State::InAttrValueNq,
    tokenizer::State::BeforeDeclaration,
    tokenizer::State::InDeclaration,
    tokenizer::State::InProcessingInstruction,
    tokenizer::State::BeforeComment,
    tokenizer::State::CDATASequence,
    tokenizer::State::InSpecialComment,
    tokenizer::State::InCommentLike,
    tokenizer::State::BeforeSpecialS,
    tokenizer::State::BeforeSpecialT,
    tokenizer::State::SpecialStartSequence,
    tokenizer::State::InRCDATA,
    tokenizer::State::InEntity,
    tokenizer::State::InSFCRootTagName,
];

const CODES_MODULE: [u8; 35] = [
    tokenizer::char_codes::TAB,
    tokenizer::char_codes::NEWLINE,
    tokenizer::char_codes::FORM_FEED,
    tokenizer::char_codes::CARRIAGE_RETURN,
    tokenizer::char_codes::SPACE,
    tokenizer::char_codes::EXCLAMATION_MARK,
    tokenizer::char_codes::DOUBLE_QUOTE,
    tokenizer::char_codes::NUMBER,
    tokenizer::char_codes::AMP,
    tokenizer::char_codes::SINGLE_QUOTE,
    tokenizer::char_codes::DASH,
    tokenizer::char_codes::DOT,
    tokenizer::char_codes::SLASH,
    tokenizer::char_codes::ZERO,
    tokenizer::char_codes::NINE,
    tokenizer::char_codes::COLON,
    tokenizer::char_codes::SEMI,
    tokenizer::char_codes::LT,
    tokenizer::char_codes::EQ,
    tokenizer::char_codes::GT,
    tokenizer::char_codes::QUESTION_MARK,
    tokenizer::char_codes::AT,
    tokenizer::char_codes::UPPER_A,
    tokenizer::char_codes::UPPER_F,
    tokenizer::char_codes::UPPER_Z,
    tokenizer::char_codes::LEFT_SQUARE,
    tokenizer::char_codes::RIGHT_SQUARE,
    tokenizer::char_codes::GRAVE_ACCENT,
    tokenizer::char_codes::LOWER_A,
    tokenizer::char_codes::LOWER_F,
    tokenizer::char_codes::LOWER_V,
    tokenizer::char_codes::LOWER_X,
    tokenizer::char_codes::LOWER_Z,
    tokenizer::char_codes::LEFT_BRACE,
    tokenizer::char_codes::RIGHT_BRACE,
];

const CODES_ROOT: [u8; 35] = [
    armature::char_codes::TAB,
    armature::char_codes::NEWLINE,
    armature::char_codes::FORM_FEED,
    armature::char_codes::CARRIAGE_RETURN,
    armature::char_codes::SPACE,
    armature::char_codes::EXCLAMATION_MARK,
    armature::char_codes::DOUBLE_QUOTE,
    armature::char_codes::NUMBER,
    armature::char_codes::AMP,
    armature::char_codes::SINGLE_QUOTE,
    armature::char_codes::DASH,
    armature::char_codes::DOT,
    armature::char_codes::SLASH,
    armature::char_codes::ZERO,
    armature::char_codes::NINE,
    armature::char_codes::COLON,
    armature::char_codes::SEMI,
    armature::char_codes::LT,
    armature::char_codes::EQ,
    armature::char_codes::GT,
    armature::char_codes::QUESTION_MARK,
    armature::char_codes::AT,
    armature::char_codes::UPPER_A,
    armature::char_codes::UPPER_F,
    armature::char_codes::UPPER_Z,
    armature::char_codes::LEFT_SQUARE,
    armature::char_codes::RIGHT_SQUARE,
    armature::char_codes::GRAVE_ACCENT,
    armature::char_codes::LOWER_A,
    armature::char_codes::LOWER_F,
    armature::char_codes::LOWER_V,
    armature::char_codes::LOWER_X,
    armature::char_codes::LOWER_Z,
    armature::char_codes::LEFT_BRACE,
    armature::char_codes::RIGHT_BRACE,
];

// Frozen values from published vize_armature 0.429.1, not from this candidate.
const PUBLISHED_CODES: [u8; 35] = [
    0x09, 0x0A, 0x0C, 0x0D, 0x20, 0x21, 0x22, 0x23, 0x26, 0x27, 0x2D, 0x2E, 0x2F, 0x30, 0x39, 0x3A,
    0x3B, 0x3C, 0x3D, 0x3E, 0x3F, 0x40, 0x41, 0x46, 0x5A, 0x5B, 0x5D, 0x60, 0x61, 0x66, 0x76, 0x78,
    0x7A, 0x7B, 0x7D,
];

// Exhaustive matches make a new public variant a compile error in this witness.
fn published_state_code(state: armature::State) -> u8 {
    match state {
        armature::State::Text => 1,
        armature::State::InterpolationOpen => 2,
        armature::State::Interpolation => 3,
        armature::State::InterpolationClose => 4,
        armature::State::BeforeTagName => 5,
        armature::State::InTagName => 6,
        armature::State::InSelfClosingTag => 7,
        armature::State::BeforeClosingTagName => 8,
        armature::State::InClosingTagName => 9,
        armature::State::AfterClosingTagName => 10,
        armature::State::BeforeAttrName => 11,
        armature::State::InTagComment => 12,
        armature::State::InAttrName => 13,
        armature::State::InDirName => 14,
        armature::State::InDirArg => 15,
        armature::State::InDirDynamicArg => 16,
        armature::State::InDirModifier => 17,
        armature::State::AfterAttrName => 18,
        armature::State::BeforeAttrValue => 19,
        armature::State::InAttrValueDq => 20,
        armature::State::InAttrValueSq => 21,
        armature::State::InAttrValueNq => 22,
        armature::State::BeforeDeclaration => 23,
        armature::State::InDeclaration => 24,
        armature::State::InProcessingInstruction => 25,
        armature::State::BeforeComment => 26,
        armature::State::CDATASequence => 27,
        armature::State::InSpecialComment => 28,
        armature::State::InCommentLike => 29,
        armature::State::BeforeSpecialS => 30,
        armature::State::BeforeSpecialT => 31,
        armature::State::SpecialStartSequence => 32,
        armature::State::InRCDATA => 33,
        armature::State::InEntity => 34,
        armature::State::InSFCRootTagName => 35,
    }
}

fn published_quote_code(quote: armature::QuoteType) -> u8 {
    match quote {
        armature::QuoteType::NoValue => 0,
        armature::QuoteType::Unquoted => 1,
        armature::QuoteType::Single => 2,
        armature::QuoteType::Double => 3,
    }
}

#[derive(Default, Clone)]
struct Sink {
    text: Rc<Cell<usize>>,
    entities: Rc<Cell<usize>>,
    interpolation: Rc<Cell<usize>>,
    errors: Rc<Cell<usize>>,
}

impl tokenizer::Callbacks for Sink {
    fn on_text(&mut self, _: usize, _: usize) {
        self.text.set(self.text.get() + 1);
    }
    fn on_text_entity(&mut self, _: char, _: usize, _: usize) {
        self.entities.set(self.entities.get() + 1);
    }
    fn on_interpolation(&mut self, _: usize, _: usize) {
        self.interpolation.set(self.interpolation.get() + 1);
    }
    fn on_open_tag_name(&mut self, _: usize, _: usize) {}
    fn on_open_tag_end(&mut self, _: usize) {}
    fn on_self_closing_tag(&mut self, _: usize) {}
    fn on_close_tag(&mut self, _: usize, _: usize) {}
    fn on_attrib_data(&mut self, _: usize, _: usize) {}
    fn on_attrib_entity(&mut self, _: char, _: usize, _: usize) {}
    fn on_attrib_end(&mut self, _: tokenizer::QuoteType, _: usize) {}
    fn on_attrib_name(&mut self, _: usize, _: usize) {}
    fn on_attrib_name_end(&mut self, _: usize) {}
    fn on_dir_name(&mut self, _: usize, _: usize) {}
    fn on_dir_arg(&mut self, _: usize, _: usize) {}
    fn on_dir_modifier(&mut self, _: usize, _: usize) {}
    fn on_comment(&mut self, _: usize, _: usize) {}
    fn on_cdata(&mut self, _: usize, _: usize) {}
    fn on_processing_instruction(&mut self, _: usize, _: usize) {}
    fn on_end(&mut self) {}
    fn on_error(&mut self, _: armature::ErrorCode, _: usize) {
        self.errors.set(self.errors.get() + 1);
    }
}

fn accepts_root_callback<C: armature::Callbacks>(_: &C) {}

pub fn exercise_old_api() {
    assert_eq!(CODES_MODULE, PUBLISHED_CODES);
    assert_eq!(CODES_ROOT, PUBLISHED_CODES);
    for (index, state) in STATES.into_iter().enumerate() {
        assert_eq!(published_state_code(state), (index + 1) as u8);
        assert_eq!(state as u8, (index + 1) as u8);
    }
    for (quote, value) in [
        (armature::QuoteType::NoValue, 0),
        (armature::QuoteType::Unquoted, 1),
        (armature::QuoteType::Single, 2),
        (armature::QuoteType::Double, 3),
    ] {
        assert_eq!(published_quote_code(quote), value);
        assert_eq!(quote as u8, value);
    }
    let mut sink = Sink::default();
    accepts_root_callback(&sink);
    // The module and crate-root names must refer to the same types.
    let _: armature::State = tokenizer::State::Text;
    let _: armature::QuoteType = tokenizer::QuoteType::Double;
    let _: armature::Tokenizer<'_, Sink> = tokenizer::Tokenizer::new("", Sink::default());
    assert_eq!(
        armature::is_whitespace(b' '),
        tokenizer::is_whitespace(b' ')
    );
    assert_eq!(
        armature::is_tag_start_char(b'A'),
        tokenizer::is_tag_start_char(b'A')
    );
    assert_eq!(
        armature::is_end_of_tag_section(b'>'),
        tokenizer::is_end_of_tag_section(b'>')
    );
    assert_eq!(armature::char_codes::AMP, tokenizer::char_codes::AMP);
    assert_eq!(armature::char_codes::TAB, 0x09);

    // Default trait methods and tokenizer methods were part of the old API.
    tokenizer::Callbacks::on_raw_interpolation(&mut sink, 0, 0);
    tokenizer::Callbacks::on_in_tag_comment(&mut sink, 0, 0);
    assert_eq!(sink.interpolation.get(), 1);
    sink.interpolation.set(0);
    let mut lexer: armature::Tokenizer<'_, Sink> = tokenizer::Tokenizer::with_delimiters(
        "<div>{{value}}x&amp;y</div>",
        sink.clone(),
        b"{{",
        b"}}",
    );
    lexer.set_tolerate_declarations(false);
    lexer.set_triple_mustache(false);
    lexer.set_in_tag_comments(false);
    lexer.tokenize();
    assert_eq!(sink.interpolation.get(), 1);
    assert!(sink.text.get() > 0);
    assert_eq!(sink.entities.get(), 1);
    assert_eq!(sink.errors.get(), 0);
}

#[cfg(test)]
mod tests {
    #[test]
    fn old_public_paths_compile_and_run() {
        super::exercise_old_api();
    }
}
