//! Private recording sink for the sole original Document lexer run.

use vize_l0::{Span, Vec};

use super::{
    DocumentLexicalError, DocumentLexicalRefusal, DocumentTokenKind as Kind, Event, RecordedKind,
};
use crate::markup::{LexErrorCode, QuoteType, Sink, entity::DecodedEntity};

pub(super) struct Recorder<'a, 'o> {
    pub events: &'o mut Vec<'a, Event>,
    pub errors: &'o mut Vec<'a, DocumentLexicalError>,
    pub end_calls: usize,
    pub declaration_refusal: Option<DocumentLexicalRefusal>,
}

impl Recorder<'_, '_> {
    fn push(&mut self, kind: Kind, start: usize, end: usize) {
        self.events.push(Event {
            kind: RecordedKind::Token(kind),
            span: Span::new(start as u32, end as u32),
        });
    }

    fn entity(&mut self, kind: RecordedKind, start: usize, end: usize) {
        self.events.push(Event {
            kind,
            span: Span::new(start as u32, end as u32),
        });
    }
}

macro_rules! ranges {
    ($($method:ident => $kind:ident),* $(,)?) => {
        $(fn $method(&mut self, start: usize, end: usize) {
            self.push(Kind::$kind, start, end);
        })*
    };
}

macro_rules! points {
    ($($method:ident => $kind:ident),* $(,)?) => {
        $(fn $method(&mut self, end: usize) {
            self.push(Kind::$kind, end, end);
        })*
    };
}

impl Sink for Recorder<'_, '_> {
    ranges! {
        on_text => Text,
        on_interpolation => Interpolation,
        on_open_tag_name => OpenTagName,
        on_close_tag => CloseTagName,
        on_attrib_name => AttributeName,
        on_attrib_data => AttributeData,
        on_dir_name => DirectiveName,
        on_dir_arg => DirectiveArgument,
        on_dir_modifier => DirectiveModifier,
        on_comment => Comment,
        on_cdata => Cdata,
        on_processing_instruction => ProcessingInstruction,
    }
    points! {
        on_open_tag_end => OpenTagEnd,
        on_self_closing_tag => SelfClosingTag,
        on_attrib_name_end => AttributeNameEnd,
    }

    fn on_text_entity(&mut self, ch: char, start: usize, end: usize) {
        self.on_text_entity_value(DecodedEntity::Numeric(ch), start, end);
    }

    fn on_text_entity_value(&mut self, value: DecodedEntity, start: usize, end: usize) {
        self.entity(RecordedKind::TextEntity(value), start, end);
    }

    fn on_attrib_entity(&mut self, ch: char, start: usize, end: usize) {
        self.on_attrib_entity_value(DecodedEntity::Numeric(ch), start, end);
    }

    fn on_attrib_entity_value(&mut self, value: DecodedEntity, start: usize, end: usize) {
        self.entity(RecordedKind::AttributeEntity(value), start, end);
    }

    fn on_attrib_end(&mut self, quote: QuoteType, end: usize) {
        self.push(Kind::AttributeEnd(quote), end, end);
    }

    fn on_declaration(&mut self, start: usize, end: usize, terminated: bool) {
        self.push(Kind::Declaration { terminated }, start, end);
        if !terminated && self.declaration_refusal.is_none() {
            self.declaration_refusal = Some(DocumentLexicalRefusal::UnterminatedDeclaration);
        }
    }

    fn on_declaration_recovery(&mut self, start: usize, end: usize, terminated: bool) {
        self.push(Kind::DeclarationRecovery { terminated }, start, end);
        self.declaration_refusal = Some(DocumentLexicalRefusal::RecoveredDeclaration);
    }

    fn on_error(&mut self, code: LexErrorCode, index: usize) {
        self.errors.push(DocumentLexicalError {
            code,
            offset: index as u32,
        });
    }

    fn on_end(&mut self) {
        self.end_calls += 1;
    }
}
