//! pug's mixins run at build time, as `pug-code-gen@3.0.4` compiles them.
//!
//! A definition runs where it stands: `pug_mixins[name] = function(params)
//! {…}` assigns a closure over the frames around it. A call
//! (`pug_mixins[name].call({block, attributes}, args)`) evaluates its
//! arguments, attributes and `block` — itself a closure over the call
//! site — in the caller's scope, then runs the body under the
//! definition's frames with `block` and `attributes` bound. The body of a
//! mixin nothing calls is only syntax-checked, as pug drops it.

use alloc::vec::Vec as StdVec;

use vize_l0::{String, cstr};
use vize_l1::pug::{PugBlock, PugCall, PugInline, PugRefused, PugTag, PugTextPiece};

use super::emit::Emitter;
use super::eval;
use super::literal::is_js_space;
use super::scope::FrameId;
use super::value::Value;

/// Nested calls deeper than this are refused (a mixin recursing on
/// build-time data stays far below it).
const MAX_DEPTH: usize = 256;

/// Calls per template, bounding mixins that expand exponentially.
const MAX_CALLS: usize = 1 << 16;

/// A defined mixin: the closure `pug_mixins[name]` holds.
pub(super) struct Mixin<'t> {
    name: String,
    params: Option<&'t str>,
    definition: &'t PugRefused<'t>,
    frame: FrameId,
}

/// A call's `block`: its content, closed over the call site.
pub(super) struct Block<'t> {
    tag: &'t PugTag<'t>,
    frame: FrameId,
}

/// The mixins a template defines and the blocks its calls pass.
#[derive(Default)]
pub(super) struct Mixins<'t> {
    defined: StdVec<Mixin<'t>>,
    blocks: StdVec<Block<'t>>,
    /// Every definition that ran, and whether anything called it.
    definitions: StdVec<(&'t PugRefused<'t>, bool)>,
    depth: usize,
    calls: usize,
}

/// `pug-lexer`'s `mixin`: `^mixin +([-\w]+)(?: *\((.*)\))? *`, where the
/// rest of the line may only be a `//-` comment pug strips.
fn head(text: &str) -> Option<(&str, Option<&str>)> {
    let rest = text.strip_prefix("mixin")?.trim_start_matches(' ');
    let name_len = rest
        .bytes()
        .take_while(|&b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
        .count();
    let (name, rest) = rest.split_at_checked(name_len)?;
    let (params, rest) = match rest
        .trim_start_matches(' ')
        .strip_prefix('(')
        .and_then(|inner| Some((inner, inner.rfind(')')?)))
    {
        Some((inner, close)) => (inner.get(..close), inner.get(close + 1..)?),
        None => (None, rest),
    };
    let rest = rest.trim_start_matches(' ').trim_end_matches(is_js_space);
    (!name.is_empty() && (rest.is_empty() || rest.starts_with("//-"))).then_some((name, params))
}

/// pug's rest parameter: the last comma-separated piece of the list, if
/// it starts with `...`, and the list without it.
fn rest_parameter(params: &str) -> (&str, Option<(&str, usize)>) {
    let pieces = params.split(',').count();
    let (before, last) = params.rsplit_once(',').unwrap_or(("", params));
    match last.trim().strip_prefix("...") {
        Some(name) => (before, Some((name, pieces - 1))),
        None => (params, None),
    }
}

/// `[a-zA-Z_$][\w$]*`.
fn is_identifier(name: &str) -> bool {
    let start = |ch: char| ch.is_ascii_alphabetic() || ch == '_' || ch == '$';
    name.starts_with(start) && name.chars().all(|ch| start(ch) || ch.is_ascii_digit())
}

/// pug's `mixin.block`: same-line content plus the indented block.
fn has_block(tag: &PugTag<'_>) -> bool {
    let inline = !matches!(tag.inline, PugInline::None | PugInline::SelfClosing(_));
    inline
        || match &tag.block {
            PugBlock::Nodes(nodes) => !nodes.is_empty(),
            PugBlock::Text(text) => text
                .pieces
                .iter()
                .any(|piece| !matches!(piece, PugTextPiece::Newline(_))),
        }
}

impl<'t> Emitter<'t> {
    /// `mixin name(params)` with its body: assign the closure.
    pub(super) fn define(&mut self, definition: &'t PugRefused<'t>) {
        let span = self.span(&definition.head);
        let Some((name, params)) = head(definition.head.text) else {
            self.error(span, String::from(super::refusal::MIXIN_HEAD));
            self.check(&definition.block);
            return;
        };
        if definition.block.is_empty() {
            self.error(span, super::refusal::without_body(name));
            return;
        }
        if name == "__proto__" {
            self.error(span, String::from(super::refusal::MIXIN_PROTO));
            return;
        }
        let frame = self.scope.capture();
        let mixin = Mixin {
            name: String::from(name),
            params,
            definition,
            frame,
        };
        match self.mixins.defined.iter_mut().find(|m| m.name == name) {
            Some(slot) => *slot = mixin,
            None => self.mixins.defined.push(mixin),
        }
        let seen = self
            .mixins
            .definitions
            .iter()
            .any(|(d, _)| core::ptr::eq(*d, definition));
        if !seen {
            self.mixins.definitions.push((definition, false));
        }
    }

    /// `+name(args)…`: run the mixin with this call's arguments,
    /// attributes and block.
    pub(super) fn call(&mut self, call: &'t PugCall<'t>) {
        let head = self.span(&call.head);
        let Some(name) = self.call_name(call) else {
            return;
        };
        let Some(index) = self.mixins.defined.iter().position(|m| m.name == name) else {
            self.error(head, super::refusal::undefined_mixin(&name));
            return;
        };
        let args = match call.arguments() {
            None => StdVec::new(),
            Some(source) => match eval::expression(&self.scope, &cstr!("[{source}]")) {
                Ok(evaluated) => match evaluated.value {
                    Value::Array(items) => items,
                    other => alloc::vec![other],
                },
                Err(reason) => {
                    let span = call.args.as_ref().map_or(head, |args| self.span(args));
                    self.error(span, super::refusal::call(&reason));
                    return;
                }
            },
        };
        let collected = self.collect(&call.tag);
        let Some(attributes) = self.call_attributes(&collected) else {
            return;
        };
        if self.mixins.depth >= MAX_DEPTH {
            self.error(head, String::from(super::refusal::MIXIN_DEPTH));
            return;
        }
        if self.mixins.calls >= MAX_CALLS {
            self.error(head, String::from(super::refusal::MIXIN_BUDGET));
            return;
        }
        self.mixins.calls += 1;
        let block = if has_block(&call.tag) {
            let frame = self.scope.capture();
            self.mixins.blocks.push(Block {
                tag: &call.tag,
                frame,
            });
            Value::Function(self.mixins.blocks.len() - 1)
        } else {
            Value::Undefined
        };
        let Some(mixin) = self.mixins.defined.get(index) else {
            return;
        };
        let (definition, params, frame) = (mixin.definition, mixin.params, mixin.frame);
        if let Some((_, called)) =
            (self.mixins.definitions.iter_mut()).find(|(d, _)| core::ptr::eq(*d, definition))
        {
            *called = true;
        }
        let caller = self.scope.enter(frame);
        match self.bind(params.unwrap_or_default(), &args, block, attributes) {
            Ok(()) => {
                self.mixins.depth += 1;
                self.nodes(&definition.block);
                self.mixins.depth -= 1;
            }
            Err(reason) => {
                let span = self.span(&definition.head);
                self.error(span, super::refusal::parameters(&name, &reason));
            }
        }
        self.scope.leave(caller);
    }

    /// The called name: the identifier, or `#{expr}`'s value.
    fn call_name(&mut self, call: &PugCall<'_>) -> Option<String> {
        let name = call.head.text.get(1..)?.trim_start_matches(is_js_space);
        let Some(source) = name
            .starts_with("#{")
            .then(|| super::control::interpolated(name))
            .flatten()
        else {
            return Some(String::from(name));
        };
        let span = self.span(&call.head);
        match eval::expression(&self.scope, source) {
            Ok(evaluated) if evaluated.value.has_function() => {
                let reason = super::refusal::FUNCTION_VALUE;
                self.error(span, super::refusal::call(reason));
                None
            }
            Ok(evaluated) => Some(evaluated.value.to_js_string()),
            Err(reason) => {
                self.error(span, super::refusal::call(&reason));
                None
            }
        }
    }

    /// The mixin function's prologue: parameters, then `var block`,
    /// `attributes` and the rest parameter.
    fn bind(
        &mut self,
        params: &str,
        args: &[Value],
        block: Value,
        attributes: Value,
    ) -> Result<(), eval::Refusal> {
        let (params, rest) = rest_parameter(params);
        eval::bind_parameters(&mut self.scope, params, args)?;
        let attributes = if attributes.truthy() {
            attributes
        } else {
            Value::Object(StdVec::new())
        };
        self.scope.declare_var("block", block)?;
        self.scope.declare_var("attributes", attributes)?;
        if let Some((name, from)) = rest {
            if !is_identifier(name) {
                return Err(cstr!("the rest parameter `...{name}` is not a plain name"));
            }
            let extra = args.get(from..).unwrap_or_default().to_vec();
            self.scope.declare_var(name, Value::Array(extra))?;
        }
        Ok(())
    }

    /// `block` in a mixin body: `block && block()`.
    pub(super) fn mixin_block(&mut self, keyword: &'t PugRefused<'t>) {
        let span = self.span(&keyword.head);
        let bare = keyword.head.text.trim_end_matches(is_js_space) == "block";
        if !bare || keyword.body.is_some() || !keyword.block.is_empty() {
            self.error(span, String::from(super::refusal::MIXIN_BLOCK_LINE));
            return;
        }
        let block = match self.scope.lookup("block") {
            Some(Value::Function(index)) => self
                .mixins
                .blocks
                .get(*index)
                .map(|block| (block.tag, block.frame)),
            Some(value) if value.truthy() => {
                self.error(span, String::from(super::refusal::MIXIN_BLOCK_VALUE));
                return;
            }
            _ => None,
        };
        let Some((tag, frame)) = block else {
            return;
        };
        if self.mixins.depth >= MAX_DEPTH {
            self.error(span, String::from(super::refusal::MIXIN_DEPTH));
            return;
        }
        // `block: function(){…}` is a function of its own.
        let caller = self.scope.enter(frame);
        self.mixins.depth += 1;
        self.content(tag);
        self.mixins.depth -= 1;
        self.scope.leave(caller);
    }

    /// Syntax-check the bodies of mixins nothing called (pug drops them,
    /// but its lexer has checked their code).
    pub(super) fn check_uncalled(&mut self) {
        let uncalled: StdVec<_> = (self.mixins.definitions.iter())
            .filter(|(_, called)| !called)
            .map(|(definition, _)| *definition)
            .collect();
        for definition in uncalled {
            self.check(&definition.block);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{head, rest_parameter};

    #[test]
    fn heads_split_like_pugs_lexer() {
        assert_eq!(head("mixin card"), Some(("card", None)));
        assert_eq!(head("mixin a-b (x, y) "), Some(("a-b", Some("x, y"))));
        // `(.*)` is greedy: the arguments run to the last `)`.
        assert_eq!(head("mixin m(a) (b)"), Some(("m", Some("a) (b"))));
        assert_eq!(head("mixin m //- note"), Some(("m", None)));
        assert_eq!(head("mixin m(a) p"), None);
        assert_eq!(head("mixin m(a"), None);
        assert_eq!(rest_parameter("a, ...items"), ("a", Some(("items", 1))));
        // pug counts comma pieces, not parameters.
        assert_eq!(
            rest_parameter("a = [1, 2], ...r"),
            ("a = [1, 2]", Some(("r", 2)))
        );
        assert_eq!(rest_parameter("a, b"), ("a, b", None));
    }
}
