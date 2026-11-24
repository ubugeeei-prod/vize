//! Template reads of setup names that no expression spells. They leave the
//! script's unused candidate relation and join a demanded read relation.

use vize_carton::CompactString;
use vize_relief::{ExpressionNode, PropNode};

use super::Drawer;
use crate::croquis::TemplateBindingReads;
use crate::scope::{ScopeData, ScopeKind};

/// The binding relations a template walk was asked for. Unused candidates
/// exist only under the unused demand, so a walk with neither demand tests
/// this one byte where main tested the candidate list, and runs main's code.
#[derive(Clone, Copy, Default)]
pub(crate) struct BindingDemand(u8);

impl BindingDemand {
    pub(crate) const NONE: Self = Self(0);
    const UNUSED: u8 = 1;
    const READS: u8 = 2;

    #[inline(always)]
    pub(crate) const fn any(self) -> bool {
        self.0 != 0
    }

    #[inline(always)]
    pub(crate) const fn unused(self) -> bool {
        self.0 & Self::UNUSED != 0
    }

    #[inline(always)]
    pub(crate) const fn reads(self) -> bool {
        self.0 & Self::READS != 0
    }

    pub(crate) fn demand_unused(&mut self) {
        self.0 |= Self::UNUSED;
    }

    pub(crate) fn demand_reads(&mut self) {
        self.0 |= Self::READS;
    }
}

impl Drawer {
    /// Whether a tag, directive or `ref` read has a relation to join.
    #[inline(always)]
    fn joins_setup_reads(&self) -> bool {
        self.binding_demand.any()
            && (self.binding_demand.reads() || !self.croquis.unused_bindings.is_empty())
    }

    /// Out of line behind the demand test, so the element walk without a
    /// binding demand inlines only that test, as on main.
    #[inline(always)]
    pub(super) fn read_setup_ref_prop(&mut self, prop: &PropNode<'_>) {
        if self.joins_setup_reads() {
            self.join_setup_ref_prop(prop);
        }
    }

    #[inline(never)]
    fn join_setup_ref_prop(&mut self, prop: &PropNode<'_>) {
        let name = match prop {
            PropNode::Attribute(attr) if attr.name == "ref" => attr
                .value
                .as_ref()
                .map(|value| CompactString::new(value.content)),
            // `<div is="vue:Foo">` resolves `Foo` the way a `<Foo>` tag does.
            PropNode::Attribute(attr) if self.binding_demand.reads() && attr.name == "is" => {
                if let Some(reads) = self.croquis.setup_context.template_binding_reads_mut()
                    && let Some(tag) = attr
                        .value
                        .as_ref()
                        .and_then(|value| value.content.strip_prefix("vue:"))
                {
                    reads.note_asset(tag.split('.').next().unwrap_or(tag));
                }
                None
            }
            PropNode::Directive(dir)
                if dir.name == "bind"
                    && matches!(&dir.arg, Some(ExpressionNode::Simple(arg)) if arg.content == "ref") =>
            {
                match &dir.exp {
                    Some(ExpressionNode::Simple(exp)) => exp
                        .js_ast
                        .and_then(|js| js.as_expression())
                        .and_then(|js| match js.ast {
                            oxc_ast::ast::Expression::StringLiteral(value) => {
                                Some(CompactString::new(value.value.as_str()))
                            }
                            _ => None,
                        }),
                    _ => None,
                }
            }
            _ => None,
        };
        if let Some(name) = name {
            self.croquis
                .unused_bindings
                .retain(|candidate| candidate.as_str() != name.as_str());
            if self.binding_demand.reads()
                && let Some(reads) = self.croquis.setup_context.template_binding_reads_mut()
            {
                reads.note_ref(&name);
            }
        }
    }

    pub(super) fn read_setup_tag(&mut self, tag: &str) {
        if !self.joins_setup_reads() {
            return;
        }
        // Vue resolves Foo, foo-bar and Foo.Bar through the setup scope.
        let root = tag.split('.').next().unwrap_or(tag);
        let normalized = TemplateBindingReads::asset_key(root);
        self.croquis
            .unused_bindings
            .retain(|name| TemplateBindingReads::asset_key(name) != normalized);
        if self.binding_demand.reads()
            && let Some(reads) = self.croquis.setup_context.template_binding_reads_mut()
        {
            reads.note_asset(&normalized);
        }
    }

    pub(super) fn read_setup_directive(&mut self, directive: &str) {
        if self.joins_setup_reads() {
            let spelling = vize_carton::cstr!("v-{directive}");
            self.read_setup_tag(&spelling);
        }
    }

    /// Expression text that does not parse proves no read absent, for either
    /// relation. Out of line, so the expression walk without a read demand
    /// stays the size it was.
    #[inline(never)]
    pub(super) fn refuse_unparsed_expression(&mut self, content: &str, unretained: bool) {
        let unused_candidates =
            self.binding_demand.unused() && !self.croquis.unused_bindings.is_empty();
        let reads_open = self
            .croquis
            .setup_context
            .template_binding_reads()
            .is_some_and(TemplateBindingReads::is_complete);
        if !(unused_candidates || reads_open)
            || !unretained
            || super::extract_identifiers_checked(content).is_some()
        {
            return;
        }
        if unused_candidates {
            self.croquis.unused_bindings.clear();
        }
        if let Some(reads) = self.croquis.setup_context.template_binding_reads_mut() {
            reads.mark_incomplete();
        }
    }

    /// A destructuring default of a `v-for` or slot pattern (`{ item = Fallback }`)
    /// evaluates in the scope around the pattern. The expression walk never
    /// visits a pattern, so a read demand joins those defaults from the
    /// scopes the walk recorded.
    #[inline(never)]
    pub(super) fn read_pattern_defaults(&mut self) {
        let Some(reads) = self.croquis.setup_context.template_binding_reads_mut() else {
            return;
        };
        let scopes = &self.croquis.scopes;
        for scope in scopes.iter() {
            let pattern = match scope.data() {
                ScopeData::VFor(data) => Some(data.value_alias.as_str()),
                ScopeData::VSlot(data) => data.props_pattern.as_deref(),
                _ => None,
            };
            let Some(pattern) = pattern.filter(|pattern| pattern.contains('=')) else {
                continue;
            };
            // The assignment form reads a destructuring target's defaults.
            let assignment = vize_carton::cstr!("({pattern} = 0)");
            let Some(names) = super::extract_identifiers_checked(&assignment) else {
                reads.mark_incomplete();
                continue;
            };
            // Template scopes around the default, the pattern's own included.
            let mut locals = Vec::new();
            let mut enclosing = Some(scope);
            while let Some(current) = enclosing {
                if matches!(
                    current.kind,
                    ScopeKind::VFor
                        | ScopeKind::VSlot
                        | ScopeKind::EventHandler
                        | ScopeKind::Callback
                        | ScopeKind::Closure
                        | ScopeKind::Block
                ) {
                    locals.extend(current.bindings().map(|(name, _)| CompactString::new(name)));
                }
                enclosing = current.parent().and_then(|parent| scopes.get_scope(parent));
            }
            reads.note_expression(&names, &locals);
        }
    }
}
