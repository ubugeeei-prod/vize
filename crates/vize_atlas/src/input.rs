//! Open typed compilation inputs for dialects, targets, and other capabilities.

use std::{
    any::{Any, TypeId},
    error::Error,
    fmt,
};
use vize_carton::FxHashMap;

use crate::{Shared, SourceId};

/// An open typed configuration or capability input.
///
/// Owning crates define markers for Vue dialects, target capabilities, feature
/// flags, or project options without adding an Atlas enum variant.
pub trait CompilationInput: Send + Sync + 'static {
    type Value: Send + Sync + 'static;
    const NAME: &'static str;
}

/// Open typed configuration attached to exactly one source identity.
///
/// Source inputs are the correct contract for per-file parser/compiler/tool
/// options: changing one source's request invalidates only artifacts whose
/// dependency closure includes that `(SourceId, SourceInputId)` pair.
pub trait SourceInput: Send + Sync + 'static {
    type Value: Send + Sync + 'static;
    const NAME: &'static str;
}

/// Runtime identity of an open [`CompilationInput`] marker.
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct InputId {
    type_id: TypeId,
    name: &'static str,
}

impl InputId {
    pub fn of<I: CompilationInput>() -> Self {
        Self {
            type_id: TypeId::of::<I>(),
            name: I::NAME,
        }
    }

    pub const fn name(self) -> &'static str {
        self.name
    }
}

impl fmt::Debug for InputId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_tuple("InputId").field(&self.name).finish()
    }
}

impl fmt::Display for InputId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.name)
    }
}

/// Runtime identity of an open [`SourceInput`] marker.
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct SourceInputId {
    type_id: TypeId,
    name: &'static str,
}

impl SourceInputId {
    pub fn of<I: SourceInput>() -> Self {
        Self {
            type_id: TypeId::of::<I>(),
            name: I::NAME,
        }
    }

    pub const fn name(self) -> &'static str {
        self.name
    }
}

impl fmt::Debug for SourceInputId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("SourceInputId")
            .field(&self.name)
            .finish()
    }
}

impl fmt::Display for SourceInputId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.name)
    }
}

type ErasedInput = Shared<dyn Any + Send + Sync>;

#[derive(Clone)]
struct InputEntry {
    revision: u64,
    value: ErasedInput,
}

/// Read-only typed input store shared by planning and provider execution.
#[derive(Clone, Default)]
pub struct CompilationInputs {
    values: FxHashMap<InputId, InputEntry>,
    source_values: FxHashMap<(SourceId, SourceInputId), InputEntry>,
}

impl CompilationInputs {
    /// Read one typed input.
    pub fn get<I: CompilationInput>(&self) -> Option<&I::Value> {
        self.values
            .get(&InputId::of::<I>())
            .and_then(|entry| entry.value.downcast_ref::<I::Value>())
    }

    pub fn contains<I: CompilationInput>(&self) -> bool {
        self.values.contains_key(&InputId::of::<I>())
    }

    /// Read one typed input for one stable source identity.
    pub fn get_source<I: SourceInput>(&self, source: SourceId) -> Option<&I::Value> {
        self.source_values
            .get(&(source, SourceInputId::of::<I>()))
            .and_then(|entry| entry.value.downcast_ref::<I::Value>())
    }

    pub fn contains_source<I: SourceInput>(&self, source: SourceId) -> bool {
        self.source_values
            .contains_key(&(source, SourceInputId::of::<I>()))
    }

    pub fn len(&self) -> usize {
        self.values.len() + self.source_values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty() && self.source_values.is_empty()
    }

    /// Current revision of a runtime input identity, or zero before it is set.
    pub fn revision(&self, input: InputId) -> u64 {
        self.values.get(&input).map_or(0, |entry| entry.revision)
    }

    /// Current revision of typed input `I`, or zero before it is set.
    pub fn revision_for<I: CompilationInput>(&self) -> u64 {
        self.revision(InputId::of::<I>())
    }

    pub fn source_revision(&self, source: SourceId, input: SourceInputId) -> u64 {
        self.source_values
            .get(&(source, input))
            .map_or(0, |entry| entry.revision)
    }

    pub fn source_revision_for<I: SourceInput>(&self, source: SourceId) -> u64 {
        self.source_revision(source, SourceInputId::of::<I>())
    }

    pub(crate) fn insert<I: CompilationInput>(
        &mut self,
        value: I::Value,
    ) -> Result<bool, CompilationInputError> {
        let input = InputId::of::<I>();
        let replaced = self.values.contains_key(&input);
        let revision = self
            .revision(input)
            .checked_add(1)
            .ok_or(CompilationInputError::GenerationExhausted)?;
        self.values.insert(
            input,
            InputEntry {
                revision,
                value: Shared::new(value),
            },
        );
        Ok(replaced)
    }

    pub(crate) fn insert_source<I: SourceInput>(
        &mut self,
        source: SourceId,
        value: I::Value,
    ) -> Result<bool, CompilationInputError> {
        let input = SourceInputId::of::<I>();
        let key = (source, input);
        let replaced = self.source_values.contains_key(&key);
        let revision = self
            .source_revision(source, input)
            .checked_add(1)
            .ok_or(CompilationInputError::GenerationExhausted)?;
        self.source_values.insert(
            key,
            InputEntry {
                revision,
                value: Shared::new(value),
            },
        );
        Ok(replaced)
    }
}

/// A typed input mutation could not advance the compilation generation.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum CompilationInputError {
    GenerationExhausted,
    SourceNotFound(SourceId),
}

impl fmt::Display for CompilationInputError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::GenerationExhausted => {
                formatter.write_str("compilation input generation is exhausted")
            }
            Self::SourceNotFound(source) => {
                write!(formatter, "source input target {source} does not exist")
            }
        }
    }
}

impl Error for CompilationInputError {}
