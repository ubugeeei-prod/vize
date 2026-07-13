//! Owned script facts needed by downstream source generators.
//!
//! These facts are projected while the authored OXC `Program` is live. They
//! deliberately contain no allocator-bound AST nodes and do not extend Atlas,
//! Croquis, or Relief with backend-specific state.

#[path = "script_generator/bridge.rs"]
mod bridge;
#[path = "script_generator/macros.rs"]
mod macros;
#[path = "script_generator/options.rs"]
mod options;

use oxc_allocator::Allocator;
use oxc_ast::ast::Program;
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_carton::{FxHashSet, String};

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
pub struct ScriptDefaultExportTargets {
    pub object: Option<(usize, usize, usize)>,
    pub class: Option<(usize, usize, usize, usize, usize)>,
    pub expr: Option<(usize, usize, usize)>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ScriptOptionsApiPropsSource {
    Object(String),
    DeferredObject(String),
    Names(Vec<String>),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ScriptOptionsFunctionKind {
    Computed,
    Method,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ScriptOptionsFunction {
    pub kind: ScriptOptionsFunctionKind,
    pub safe_name: String,
    pub params: String,
    pub body: String,
}

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct ScriptOptionsApiBridge {
    pub computed: Vec<ScriptOptionsFunction>,
    pub methods: Vec<ScriptOptionsFunction>,
    pub mapped_types: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct SfcScriptGeneratorFacts {
    module_statement_spans: Vec<(u32, u32)>,
    synthetic_source_len: usize,
    source_block_count: usize,
    named_value_exports: Vec<String>,
    const_enum_names: FxHashSet<String>,
    type_references: FxHashSet<String>,
    value_references: FxHashSet<String>,
    default_export_targets: ScriptDefaultExportTargets,
    options_api_props: Option<ScriptOptionsApiPropsSource>,
    options_api_bridge: Option<ScriptOptionsApiBridge>,
    unresolved_options_extends: bool,
    props_const_assertion_offsets: Vec<usize>,
    options_setup_return_has_spread: bool,
    define_props_result_bindings: FxHashSet<String>,
    define_props_boolean_keys: Option<Vec<String>>,
}

impl SfcScriptGeneratorFacts {
    pub(crate) fn from_program(program: &Program<'_>, source: &str) -> Self {
        let usage = macros::identifier_usage(program);
        Self {
            module_statement_spans: macros::module_statement_spans(program),
            synthetic_source_len: source.len(),
            source_block_count: 1,
            named_value_exports: macros::named_value_exports(program),
            const_enum_names: macros::const_enum_names(program),
            type_references: usage.type_references,
            value_references: usage.value_references,
            default_export_targets: options::default_export_targets(program),
            options_api_props: options::options_api_props(program, source),
            options_api_bridge: bridge::options_api_bridge(program, source),
            unresolved_options_extends: options::has_unresolved_extends(program),
            props_const_assertion_offsets: options::props_const_assertion_offsets(program),
            options_setup_return_has_spread: options::setup_return_has_spread(program),
            define_props_result_bindings: macros::define_props_result_bindings(program),
            define_props_boolean_keys: macros::define_props_boolean_keys(program),
        }
    }

    /// Compatibility projection for callers outside the parse-once SFC graph.
    #[doc(hidden)]
    pub fn from_source(source: &str) -> Self {
        let allocator = Allocator::default();
        let parsed = Parser::new(&allocator, source, SourceType::tsx().with_module(true)).parse();
        if parsed.panicked {
            return Self {
                synthetic_source_len: source.len(),
                source_block_count: 1,
                ..Self::default()
            };
        }
        Self::from_program(&parsed.program, source)
    }

    pub(crate) fn merge(&mut self, other: Self) {
        let separator_len =
            usize::from(self.source_block_count > 0 && other.source_block_count > 0);
        let rebase = self.synthetic_source_len.saturating_add(separator_len);
        let rebase = u32::try_from(rebase).unwrap_or(u32::MAX);
        self.module_statement_spans.extend(
            other
                .module_statement_spans
                .into_iter()
                .map(|(start, end)| (start.saturating_add(rebase), end.saturating_add(rebase))),
        );
        self.synthetic_source_len = self
            .synthetic_source_len
            .saturating_add(separator_len)
            .saturating_add(other.synthetic_source_len);
        self.source_block_count = self
            .source_block_count
            .saturating_add(other.source_block_count);
        push_unique(&mut self.named_value_exports, other.named_value_exports);
        self.const_enum_names.extend(other.const_enum_names);
        self.type_references.extend(other.type_references);
        self.value_references.extend(other.value_references);
        if self.default_export_targets == ScriptDefaultExportTargets::default() {
            self.default_export_targets = other.default_export_targets;
        }
        self.options_api_props = self.options_api_props.take().or(other.options_api_props);
        self.options_api_bridge = self.options_api_bridge.take().or(other.options_api_bridge);
        self.unresolved_options_extends |= other.unresolved_options_extends;
        self.props_const_assertion_offsets
            .extend(other.props_const_assertion_offsets);
        self.props_const_assertion_offsets.sort_unstable();
        self.props_const_assertion_offsets.dedup();
        self.options_setup_return_has_spread |= other.options_setup_return_has_spread;
        self.define_props_result_bindings
            .extend(other.define_props_result_bindings);
        merge_optional_names(
            &mut self.define_props_boolean_keys,
            other.define_props_boolean_keys,
        );
    }

    pub fn named_value_exports(&self) -> &[String] {
        &self.named_value_exports
    }

    /// Import and source-bearing re-export spans in the synthetic script view.
    pub fn module_statement_spans(&self) -> &[(u32, u32)] {
        &self.module_statement_spans
    }

    /// Byte length of the `script + "\n" + script-setup` view represented here.
    pub const fn synthetic_source_len(&self) -> usize {
        self.synthetic_source_len
    }

    pub fn const_enum_names(&self) -> &FxHashSet<String> {
        &self.const_enum_names
    }

    pub fn type_references(&self) -> &FxHashSet<String> {
        &self.type_references
    }

    pub fn value_references(&self) -> &FxHashSet<String> {
        &self.value_references
    }

    pub const fn default_export_targets(&self) -> ScriptDefaultExportTargets {
        self.default_export_targets
    }

    pub fn options_api_props(&self) -> Option<&ScriptOptionsApiPropsSource> {
        self.options_api_props.as_ref()
    }

    pub fn options_api_bridge(&self) -> Option<&ScriptOptionsApiBridge> {
        self.options_api_bridge.as_ref()
    }

    pub const fn has_unresolved_options_extends(&self) -> bool {
        self.unresolved_options_extends
    }

    pub fn props_const_assertion_offsets(&self) -> &[usize] {
        &self.props_const_assertion_offsets
    }

    pub const fn options_setup_return_has_spread(&self) -> bool {
        self.options_setup_return_has_spread
    }

    pub fn define_props_result_bindings(&self) -> &FxHashSet<String> {
        &self.define_props_result_bindings
    }

    pub fn define_props_boolean_keys(&self) -> Option<&[String]> {
        self.define_props_boolean_keys.as_deref()
    }
}

fn push_unique(target: &mut Vec<String>, source: Vec<String>) {
    for name in source {
        if !target.contains(&name) {
            target.push(name);
        }
    }
}

fn merge_optional_names(target: &mut Option<Vec<String>>, source: Option<Vec<String>>) {
    let Some(source) = source else { return };
    let target = target.get_or_insert_default();
    push_unique(target, source);
    target.sort_unstable();
}
