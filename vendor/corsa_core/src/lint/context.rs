use super::{
    LintDiagnostic, LintNode, LintSuggestion, NodeMetadataDepth, RuleBridgeRequirements,
    RuleMessage, RuleMeta, TextRange,
};

/// A Rust-authored type-aware lint rule.
///
/// The host adapter owns AST traversal and type lookups, then sends compact
/// [`LintNode`] facts into the Rust rule. This keeps the final public surface as
/// an Oxlint JS plugin while allowing common rules to live on the Rust hot path.
pub trait RustLintRule: Send + Sync {
    /// Returns the stable rule name exposed to JavaScript and diagnostics.
    fn name(&self) -> &'static str;

    /// Returns the short prose description used in generated rule metadata.
    fn docs_description(&self) -> &'static str;

    /// Returns the message catalog keyed by `message_id`.
    fn messages(&self) -> &'static [RuleMessage];

    /// Returns the AST node kinds this rule wants the host to send.
    fn listeners(&self) -> &'static [&'static str];

    /// Returns whether diagnostics from this rule may include suggested fixes.
    fn has_suggestions(&self) -> bool {
        false
    }

    /// Returns whether the host should attach TypeScript-rendered type text.
    fn requires_type_texts(&self) -> bool {
        true
    }

    /// Returns host metadata requirements for the JavaScript native bridge.
    fn bridge_requirements(&self) -> RuleBridgeRequirements {
        bridge_requirements_for_rule(self.name(), self.requires_type_texts())
    }

    /// Checks one host-provided node and records diagnostics in `ctx`.
    fn check(&self, ctx: &mut RuleContext<'_>, node: &LintNode);

    /// Builds the serializable rule metadata exposed to binding layers.
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: self.name().to_owned(),
            docs_description: self.docs_description().to_owned(),
            messages: self
                .messages()
                .iter()
                .map(|message| (message.id.to_owned(), message.description.to_owned()))
                .collect(),
            has_suggestions: self.has_suggestions(),
            listeners: self
                .listeners()
                .iter()
                .map(|listener| (*listener).to_owned())
                .collect(),
            requires_type_texts: self.requires_type_texts(),
            bridge: self.bridge_requirements(),
        }
    }
}

fn bridge_requirements_for_rule(name: &str, requires_type_texts: bool) -> RuleBridgeRequirements {
    match name {
        "consistent-return"
        | "consistent-type-exports"
        | "dot-notation"
        | "no-unnecessary-qualifier"
        | "no-unnecessary-template-expression"
        | "no-unnecessary-type-parameters"
        | "no-useless-default-assignment"
        | "prefer-optional-chain" => RuleBridgeRequirements::syntax_only(5),
        "no-deprecated" => RuleBridgeRequirements::syntax_only(5).with_symbol_facts(),
        "no-duplicate-type-constituents" | "no-redundant-type-constituents" => {
            RuleBridgeRequirements::syntax_only(5).with_text(NodeMetadataDepth::through(1))
        }
        "no-confusing-void-expression"
        | "no-unnecessary-condition"
        | "no-unsafe-enum-comparison"
        | "prefer-nullish-coalescing"
        | "strict-boolean-expressions"
        | "switch-exhaustiveness-check"
        | "unbound-method" => RuleBridgeRequirements::type_texts(3, NodeMetadataDepth::through(1)),
        "no-misused-promises"
        | "no-misused-spread"
        | "no-unnecessary-boolean-literal-compare"
        | "no-unnecessary-type-arguments"
        | "no-unnecessary-type-assertion"
        | "no-unnecessary-type-conversion"
        | "no-unsafe-argument"
        | "non-nullable-type-assertion-style"
        | "prefer-readonly"
        | "prefer-readonly-parameter-types"
        | "prefer-return-this-type"
        | "promise-function-async"
        | "related-getter-setter-pairs"
        | "return-await"
        | "strict-void-return" => {
            RuleBridgeRequirements::type_texts(5, NodeMetadataDepth::through(2))
        }
        "require-await" => RuleBridgeRequirements::type_texts(5, NodeMetadataDepth::through(3)),
        "await-thenable" => {
            RuleBridgeRequirements::type_texts_and_properties(3, NodeMetadataDepth::range(1, 2))
        }
        "no-array-delete" => RuleBridgeRequirements::type_texts(2, NodeMetadataDepth::exact(2)),
        "no-base-to-string" => {
            RuleBridgeRequirements::type_texts(2, NodeMetadataDepth::range(1, 2))
        }
        "no-floating-promises" => {
            RuleBridgeRequirements::type_texts_and_properties(4, NodeMetadataDepth::range(1, 2))
        }
        "no-for-in-array" => RuleBridgeRequirements::type_texts(1, NodeMetadataDepth::exact(1)),
        "no-implied-eval" => RuleBridgeRequirements::type_texts(2, NodeMetadataDepth::exact(1)),
        "no-meaningless-void-operator"
        | "no-unsafe-call"
        | "no-unsafe-member-access"
        | "no-unsafe-return"
        | "no-unsafe-type-assertion"
        | "no-unsafe-unary-minus"
        | "restrict-plus-operands" => {
            RuleBridgeRequirements::type_texts(1, NodeMetadataDepth::exact(1))
        }
        "no-unsafe-assignment" => {
            RuleBridgeRequirements::type_texts(2, NodeMetadataDepth::exact(1))
        }
        "only-throw-error" | "prefer-promise-reject-errors" => {
            RuleBridgeRequirements::type_texts_and_properties(2, NodeMetadataDepth::range(1, 2))
        }
        "prefer-reduce-type-parameter" => {
            RuleBridgeRequirements::type_texts(3, NodeMetadataDepth::exact(2))
        }
        "prefer-string-starts-ends-with" => {
            RuleBridgeRequirements::type_texts(4, NodeMetadataDepth::range(2, 3))
        }
        "require-array-sort-compare" => {
            RuleBridgeRequirements::type_texts(2, NodeMetadataDepth::exact(2))
        }
        _ => RuleBridgeRequirements::default_for_type_texts(requires_type_texts),
    }
}

/// Per-node diagnostic sink passed to a [`RustLintRule`].
///
/// A context borrows the rule being executed so it can resolve message IDs into
/// stable text while collecting diagnostics for the current node.
pub struct RuleContext<'a> {
    rule: &'a dyn RustLintRule,
    diagnostics: Vec<LintDiagnostic>,
}

impl<'a> RuleContext<'a> {
    pub(crate) fn new(rule: &'a dyn RustLintRule) -> Self {
        Self {
            rule,
            diagnostics: Vec::new(),
        }
    }

    /// Reports a diagnostic without suggestions.
    pub fn report(&mut self, message_id: &'static str, range: TextRange) {
        self.report_with_suggestions(message_id, range, Vec::new());
    }

    /// Reports a diagnostic with zero or more suggested fixes.
    pub fn report_with_suggestions(
        &mut self,
        message_id: &'static str,
        range: TextRange,
        suggestions: Vec<LintSuggestion>,
    ) {
        self.diagnostics.push(LintDiagnostic {
            rule_name: self.rule.name().to_owned(),
            message_id: message_id.to_owned(),
            message: self.message(message_id),
            range,
            suggestions,
        });
    }

    pub(crate) fn finish(self) -> Vec<LintDiagnostic> {
        self.diagnostics
    }

    fn message(&self, message_id: &str) -> String {
        self.rule
            .messages()
            .iter()
            .find(|message| message.id == message_id)
            .map(|message| message.description)
            .unwrap_or(message_id)
            .to_owned()
    }
}
