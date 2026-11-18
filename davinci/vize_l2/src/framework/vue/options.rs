//! Options API facts of a normal `<script>`, observed inside the single L2
//! Program walk.
//!
//! The component is the `export default` options object, reached through
//! `defineComponent(..)`/`Vue.extend(..)` calls, TypeScript expression
//! wrappers and same-file component names, or a class component (see
//! [`class`]). Its members are read per option group in a fixed order
//! (props, computed, filters, methods, inject, data, asyncData, setup), then
//! the members of same-file `mixins` and `extends` targets; a mixin name is
//! read once. A group's value may be a same-file object name, and `data`,
//! `asyncData` and `setup` contribute the keys of the object they return.
//! A member spelled twice is listed twice, in reading order. Nothing is
//! resolved across files.

mod class;
mod read;

use crate::file::{ScopeId, ScriptUnit};
use crate::lang::js::{CallEvent, DeclaredEvent, FileObserver, StatementEvent};
use crate::resolution::ResolutionErrorKind;
use alloc::vec::Vec;
use read::{Entry, Export, Group, Object, Reads, Target, Value};
use vize_l0::Span;

/// How the component is declared.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentShape {
    Options,
    Class,
}

/// The option group, or the class member, that declares a member.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptionMemberKind {
    /// `props`, or a field decorated `@Prop` or `@Model`.
    Prop,
    /// `inject`, or a field decorated `@Inject` or `@InjectReactive`.
    Inject,
    /// `computed`, a class accessor, or a field decorated `@PropSync`,
    /// `@ModelSync` or `@VModel` (the computed wrapper over a prop of
    /// another name).
    Computed,
    /// `methods`, or a class method.
    Method,
    /// Vue 2 `filters`.
    Filter,
    /// A key of the object `data` returns (or is), or an undecorated field.
    Data,
    /// A key of the object Nuxt 2 `asyncData` returns.
    AsyncData,
    /// A key of the object `setup` returns.
    Setup,
}

/// One declared member. Spans are authored file coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OptionMember {
    pub kind: OptionMemberKind,
    /// The name: an identifier key, or a string literal's contents.
    pub name: Span,
    /// The key or array element as written, quotes included.
    pub key: Span,
    /// The options object or class declaring the member: the component's
    /// own, a decorator's, or a mixin or `extends` target.
    pub owner: Span,
}

/// A top-level key of an options object.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OptionKey {
    /// An identifier key, or a string literal's contents.
    pub name: Span,
    /// The key as written, quotes included.
    pub key: Span,
}

/// The component a normal `<script>` declares.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentOptions {
    pub shape: ComponentShape,
    /// The options object, or the class.
    pub declaration: Span,
    /// Every static top-level key of the options object, in source order,
    /// unknown ones included; none for a class.
    pub option_keys: Vec<OptionKey>,
    /// Members in reading order.
    pub members: Vec<OptionMember>,
    /// String keys in the objects read whose contents are not their source
    /// text; their members are not listed.
    pub escaped: Vec<Span>,
}

/// The facts of one normal `<script>` Program.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OptionsFacts {
    component: Option<ComponentOptions>,
}

impl OptionsFacts {
    /// The component the `export default` declares, when it declares one.
    #[must_use]
    pub fn component(&self) -> Option<&ComponentOptions> {
        self.component.as_ref()
    }
}

/// Why the observed facts do not cover the whole Program.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptionsFactsIncomplete {
    NoProgram,
    SecondProgram,
    MissingStatements { observed: usize, expected: usize },
    SpanOverflow,
}

/// Collects the Options API facts of one normal `<script>` Program from its
/// top-level statement events. Pass it to `FileProducer::program_observed`,
/// then [`Self::finish`].
#[derive(Debug, Default)]
pub struct OptionsObserver<'a> {
    program: Option<Program>,
    second_program: bool,
    overflow: bool,
    observed: usize,
    reads: Reads<'a>,
}

#[derive(Debug, Clone, Copy)]
struct Program {
    scope: ScopeId,
    base: u32,
    statements: usize,
}

impl<'a> OptionsObserver<'a> {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The facts, once every top-level statement was seen.
    pub fn finish(self) -> Result<OptionsFacts, OptionsFactsIncomplete> {
        let Some(program) = self.program else {
            return Err(OptionsFactsIncomplete::NoProgram);
        };
        if self.second_program {
            return Err(OptionsFactsIncomplete::SecondProgram);
        }
        if self.overflow {
            return Err(OptionsFactsIncomplete::SpanOverflow);
        }
        if self.observed != program.statements {
            return Err(OptionsFactsIncomplete::MissingStatements {
                observed: self.observed,
                expected: program.statements,
            });
        }
        Ok(OptionsFacts {
            component: Resolver::new(&self.reads).component(),
        })
    }
}

/// Same-file names: object literals (the last declarator of a name wins)
/// and component options (the first that resolves wins).
struct Resolver<'r, 'a> {
    reads: &'r Reads<'a>,
    objects: Vec<(&'a str, usize)>,
    components: Vec<(&'a str, usize)>,
}

impl<'r, 'a> Resolver<'r, 'a> {
    fn new(reads: &'r Reads<'a>) -> Self {
        let mut objects: Vec<(&'a str, usize)> = Vec::new();
        for declarator in &reads.declarators {
            if let Some(index) = declarator.object {
                objects.retain(|(name, _)| *name != declarator.name);
                objects.push((declarator.name, index));
            }
        }
        let mut components: Vec<(&'a str, usize)> = Vec::new();
        let mut changed = true;
        while changed {
            changed = false;
            for declarator in &reads.declarators {
                if components.iter().any(|(name, _)| *name == declarator.name) {
                    continue;
                }
                let resolved = match declarator.options {
                    Some(Target::Object(index)) => Some(index),
                    Some(Target::Binding(name)) => lookup(&components, name),
                    None => None,
                };
                if let Some(index) = resolved {
                    components.push((declarator.name, index));
                    changed = true;
                }
            }
        }
        Self {
            reads,
            objects,
            components,
        }
    }

    fn component(&self) -> Option<ComponentOptions> {
        let new = |shape, declaration| ComponentOptions {
            shape,
            declaration,
            option_keys: Vec::new(),
            members: Vec::new(),
            escaped: Vec::new(),
        };
        match self.reads.exports.first()? {
            Export::Class(class) => {
                let mut component = new(ComponentShape::Class, class.span);
                for &options in &class.options {
                    self.options(options, &mut Vec::new(), &mut component);
                }
                for &(kind, entry) in &class.members {
                    push(&mut component, kind, entry, class.span);
                }
                Some(component)
            }
            Export::Options(target) => {
                let index = match (*target)? {
                    Target::Object(index) => index,
                    Target::Binding(name) => lookup(&self.components, name)?,
                };
                let object = self.reads.objects.get(index)?;
                let mut component = new(ComponentShape::Options, object.span);
                for entry in &object.keys {
                    match *entry {
                        Entry::Key(key) => component.option_keys.push(OptionKey {
                            name: key.name,
                            key: key.key,
                        }),
                        Entry::Escaped(span) => component.escaped.push(span),
                    }
                }
                self.options(index, &mut Vec::new(), &mut component);
                Some(component)
            }
        }
    }

    /// Reads one options object's groups, then its mixins and `extends`.
    fn options(&self, index: usize, seen: &mut Vec<&'a str>, component: &mut ComponentOptions) {
        let reads: &'r Reads<'a> = self.reads;
        let Some(object) = reads.objects.get(index) else {
            return;
        };
        let groups = [
            (Group::Props, OptionMemberKind::Prop),
            (Group::Computed, OptionMemberKind::Computed),
            (Group::Filters, OptionMemberKind::Filter),
            (Group::Methods, OptionMemberKind::Method),
            (Group::Inject, OptionMemberKind::Inject),
            (Group::Data, OptionMemberKind::Data),
            (Group::AsyncData, OptionMemberKind::AsyncData),
            (Group::Setup, OptionMemberKind::Setup),
        ];
        for (group, kind) in groups {
            for &entry in self.group(object, group).unwrap_or_default() {
                push(component, kind, entry, object.span);
            }
        }
        for group in [Group::Mixins, Group::Extends] {
            let Some((_, Value::Targets(targets))) =
                object.options.iter().find(|(of, _)| *of == group)
            else {
                continue;
            };
            for &target in targets {
                let index = match target {
                    Target::Object(index) => Some(index),
                    Target::Binding(name) if !seen.contains(&name) => {
                        seen.push(name);
                        lookup(&self.objects, name)
                    }
                    Target::Binding(_) => None,
                };
                if let Some(index) = index {
                    self.options(index, seen, component);
                }
            }
        }
    }

    /// The keys a group contributes. `props`, `inject`, `data`, `asyncData`
    /// and `setup` read their first property; `computed`, `methods` and
    /// `filters` the first whose value is an object.
    fn group(&self, object: &'r Object<'a>, group: Group) -> Option<&'r [Entry]> {
        let mut values = object
            .options
            .iter()
            .filter(move |(of, _)| *of == group)
            .map(|(_, value)| value);
        match group {
            Group::Computed | Group::Methods | Group::Filters => {
                values.find_map(|value| self.value(value))
            }
            _ => values.next().and_then(|value| self.value(value)),
        }
    }

    fn value(&self, value: &'r Value<'a>) -> Option<&'r [Entry]> {
        match value {
            Value::Keys(entries) => Some(entries.as_slice()),
            Value::Binding(name) => self.keys(Target::Binding(name)),
            Value::Returned(candidates) => candidates
                .iter()
                .find_map(|candidate| self.keys(*candidate)),
            Value::Targets(_) | Value::Unread => None,
        }
    }

    fn keys(&self, target: Target<'a>) -> Option<&'r [Entry]> {
        let index = match target {
            Target::Object(index) => index,
            Target::Binding(name) => lookup(&self.objects, name)?,
        };
        let reads: &'r Reads<'a> = self.reads;
        reads
            .objects
            .get(index)
            .map(|object| object.keys.as_slice())
    }
}

fn lookup(names: &[(&str, usize)], name: &str) -> Option<usize> {
    names
        .iter()
        .find(|(declared, _)| *declared == name)
        .map(|(_, index)| *index)
}

fn push(component: &mut ComponentOptions, kind: OptionMemberKind, entry: Entry, owner: Span) {
    match entry {
        Entry::Key(key) => component.members.push(OptionMember {
            kind,
            name: key.name,
            key: key.key,
            owner,
        }),
        Entry::Escaped(span) => component.escaped.push(span),
    }
}

impl<'a> FileObserver<'a> for OptionsObserver<'a> {
    type Checkpoint = OptionsCheckpoint;

    fn unit(&mut self, unit: &ScriptUnit) {
        if self.program.is_some() {
            self.second_program = true;
            return;
        }
        self.program = Some(Program {
            scope: unit.scope,
            base: unit.span.start,
            statements: unit.origin.length,
        });
    }

    fn statement(&mut self, event: StatementEvent<'_, 'a>) {
        let Some(program) = self.program else {
            return;
        };
        if self.second_program || event.scope != program.scope {
            return;
        }
        self.observed += 1;
        let overflow = &mut self.overflow;
        let base = program.base;
        self.reads
            .statement(event.statement, &mut |span: oxc_span::Span| match (
                base.checked_add(span.start),
                base.checked_add(span.end),
            ) {
                (Some(start), Some(end)) => Span::new(start, end),
                _ => {
                    *overflow = true;
                    Span::new(0, 0)
                }
            });
    }

    fn declared(&mut self, _: DeclaredEvent<'_, 'a>) {}

    fn checkpoint(&self) -> OptionsCheckpoint {
        OptionsCheckpoint {
            observed: self.observed,
            reads: self.reads.len(),
        }
    }

    fn call(&mut self, _: CallEvent<'_, 'a>) -> Result<(), ResolutionErrorKind> {
        Ok(())
    }

    /// A rolled-back statement no longer counts as observed.
    fn rollback(&mut self, checkpoint: OptionsCheckpoint) {
        self.observed = checkpoint.observed;
        self.reads.truncate(checkpoint.reads);
    }
}

/// Row counts before a walk subtree; statement facts never straddle one.
#[derive(Debug, Clone, Copy)]
pub struct OptionsCheckpoint {
    observed: usize,
    reads: [usize; 3],
}
