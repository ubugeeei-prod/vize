use super::{Property, RuleRun, ScssProgram, ScssRefusal, Variable, VariableUse, syntax};
use alloc::{collections::BTreeMap, vec::Vec};
use vize_l0::{Span, String};
use vize_l1::css::{
    StyleNodeKind,
    lang::{OriginalStyleStructure, StyleLanguage},
};

struct Selector {
    text: String,
    span: Span,
    group: usize,
}

struct Variables<'a> {
    visible: BTreeMap<String, usize>,
    rows: Vec<Variable<'a>>,
    uses: Vec<VariableUse>,
}

impl<'a> Variables<'a> {
    fn atom(&mut self, word: &'a str, span: Span) -> Result<&'a str, ScssRefusal> {
        if let Some(name) = word
            .strip_prefix('$')
            .filter(|name| syntax::identifier(name))
        {
            let declaration = *self
                .visible
                .get(&syntax::variable_key(name))
                .ok_or(ScssRefusal::MissingVariable(span))?;
            let variable = self
                .rows
                .get(declaration)
                .ok_or(ScssRefusal::MissingVariable(span))?;
            self.uses.push(VariableUse { span, declaration });
            Ok(variable.value)
        } else if syntax::literal(word) {
            Ok(word)
        } else {
            Err(ScssRefusal::Value(span))
        }
    }

    fn declare(
        &mut self,
        name: &'a str,
        span: Span,
        value: &'a str,
        value_span: Span,
    ) -> Result<(), ScssRefusal> {
        let variable = name
            .strip_prefix('$')
            .filter(|name| syntax::identifier(name))
            .ok_or(ScssRefusal::Statement(span))?;
        let value = self.atom(value, value_span)?;
        self.visible
            .insert(syntax::variable_key(variable), self.rows.len());
        self.rows.push(Variable { name, span, value });
        Ok(())
    }

    fn value(
        &mut self,
        text: &'a str,
        span: Span,
        budget: &mut usize,
    ) -> Result<String, ScssRefusal> {
        let mut result = String::default();
        for word in text.split_ascii_whitespace() {
            let start = span.start + (word.as_ptr() as usize - text.as_ptr() as usize) as u32;
            let part = self.atom(word, Span::new(start, start + word.len() as u32))?;
            let space = usize::from(!result.is_empty());
            *budget = budget
                .checked_sub(part.len() + space)
                .ok_or(ScssRefusal::Expansion(span))?;
            if space != 0 {
                result.push(' ');
            }
            result.push_str(part);
        }
        if result.is_empty() {
            Err(ScssRefusal::Value(span))
        } else {
            Ok(result)
        }
    }
}

pub(super) fn program<'a>(
    style: OriginalStyleStructure<'_, 'a>,
) -> Result<ScssProgram<'a>, ScssRefusal> {
    if style.language() != StyleLanguage::Scss {
        return Err(ScssRefusal::Language);
    }
    let sheet = style.sheet();
    if !sheet.is_balanced() || !sheet.is_terminated() {
        return Err(ScssRefusal::IncompleteStructure);
    }
    if let Some(comment) = sheet.comments().first() {
        return Err(ScssRefusal::Comment(comment.span()));
    }
    let mut variables = Variables {
        visible: BTreeMap::new(),
        rows: Vec::new(),
        uses: Vec::new(),
    };
    let mut selectors: Vec<Option<Selector>> = Vec::with_capacity(sheet.nodes().len());
    let mut runs: Vec<RuleRun<'a>> = Vec::new();
    let mut previous = None;
    // Bound cumulative selector copies and repeated value expansions before
    // publishing the model. Nested selectors may otherwise grow quadratically.
    let mut budget = sheet.source().len().saturating_mul(32).max(256);
    for (index, node) in sheet.nodes().iter().enumerate() {
        let owner = node.parent();
        let parent = owner
            .and_then(|id| selectors.get(id))
            .and_then(Option::as_ref);
        let span = node.prelude();
        let text = sheet.text(span);
        if node.is_interpolated() {
            return Err(ScssRefusal::Statement(span));
        }
        match sheet.kind(node) {
            StyleNodeKind::Rule if node.block().is_some() => {
                let selector = syntax::selector(text, parent.map(|p| p.text.as_str()))
                    .ok_or(ScssRefusal::Selector(span))?;
                budget = budget
                    .checked_sub(selector.len())
                    .ok_or(ScssRefusal::Expansion(span))?;
                let group = parent.map_or(index, |p| p.group);
                selectors.push(Some(Selector {
                    text: selector,
                    span,
                    group,
                }));
                previous = None;
            }
            StyleNodeKind::Declaration { name, value } if node.block().is_none() => {
                let property = sheet.text(name);
                let raw = sheet.text(value);
                if let Some(parent) = parent {
                    if !syntax::identifier(property) {
                        return Err(ScssRefusal::Statement(name));
                    }
                    let resolved = variables.value(raw, value, &mut budget)?;
                    if previous != owner {
                        budget = budget
                            .checked_sub(parent.text.len())
                            .ok_or(ScssRefusal::Expansion(span))?;
                        runs.push(RuleRun {
                            selector: parent.text.clone(),
                            selector_span: parent.span,
                            group: parent.group,
                            declarations: Vec::new(),
                        });
                    }
                    let run = runs.last_mut().ok_or(ScssRefusal::Statement(span))?;
                    run.declarations.push(Property {
                        name: property,
                        value: resolved,
                        name_span: name,
                        value_span: value,
                    });
                    previous = owner;
                } else {
                    variables.declare(property, name, raw, value)?;
                }
                selectors.push(None);
            }
            _ => return Err(ScssRefusal::Statement(span)),
        }
    }
    Ok(ScssProgram {
        runs,
        variables: variables.rows,
        uses: variables.uses,
    })
}
