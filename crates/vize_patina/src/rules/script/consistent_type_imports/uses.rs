//! How every block of the component uses one import binding.
//!
//! The linted block's own references come from its resolved symbol. The other
//! script block of an SFC shares the module scope, so a name it leaves
//! unresolved is a reference to this block's import. The template side reads
//! names through the SFC's binding analysis. Where a fact is unavailable the
//! whole block is left alone: no use can be ruled out.

use oxc_ast::ast::Program;
use oxc_semantic::{ReferenceId, Semantic, SymbolId};
use oxc_syntax::reference::ReferenceFlags;
use vize_l0::{CompactString, FxHashSet};

use super::metadata;
use crate::linter::sfc_scripts::BlockSemantic;
use crate::rules::script::{SfcScriptContext, TemplateSideReads, UnboundRead};

/// The uses one import binding has anywhere in the component.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(super) struct Uses {
    /// Some use needs the binding at runtime.
    pub(super) value: bool,
    /// Some use is in a type position.
    pub(super) ty: bool,
}

/// One script block's semantic pass and the references a decorator-metadata
/// emit would read as values.
struct Block<'a> {
    built: BlockSemantic<'a, 'a>,
    metadata: FxHashSet<ReferenceId>,
}

impl<'a> Block<'a> {
    /// `None` when the block's semantics cannot prove a use absent.
    fn new(built: BlockSemantic<'a, 'a>, emit_decorator_metadata: bool) -> Option<Self> {
        if !built.diagnostics.is_empty() {
            return None;
        }
        let source = built.semantic.source_text();
        // Direct `eval` can read any binding of the module.
        let scoping = built.semantic.scoping();
        if scoping
            .scope_descendants_from_root()
            .any(|scope| scoping.scope_flags(scope).contains_direct_eval())
        {
            return None;
        }
        let metadata = if emit_decorator_metadata && source.contains('@') {
            metadata::references(&built.semantic)
        } else {
            FxHashSet::default()
        };
        Some(Self { built, metadata })
    }

    fn semantic(&self) -> &Semantic<'a> {
        &self.built.semantic
    }

    fn note(&self, reference: ReferenceId, flags: ReferenceFlags, uses: &mut Uses) {
        // An export specifier of a name another block declares stays
        // `Read | Type`: it re-exports whatever the name is, so it is a value.
        if flags.is_value() || self.metadata.contains(&reference) {
            uses.value = true;
        } else {
            uses.ty = true;
        }
    }

    /// Uses through the symbol this block declares.
    fn symbol_uses(&self, symbol: SymbolId, uses: &mut Uses) {
        let scoping = self.semantic().scoping();
        for &reference in scoping.get_resolved_reference_ids(symbol) {
            self.note(reference, scoping.get_reference(reference).flags(), uses);
        }
    }

    /// Uses of a name this block leaves to the module scope.
    fn unresolved_uses(&self, name: &str, uses: &mut Uses) {
        let scoping = self.semantic().scoping();
        let Some(references) = scoping.root_unresolved_references().get(name) else {
            return;
        };
        for &reference in references {
            self.note(reference, scoping.get_reference(reference).flags(), uses);
        }
    }
}

/// Every source of a use of the linted block's imports.
pub(super) struct ComponentUses<'a> {
    own: Block<'a>,
    sibling: Option<Block<'a>>,
    template: TemplateSideReads<'a>,
    /// The setup `generic` attribute, parsed when a binding has no other use.
    generic: Option<&'a str>,
    generic_types: Option<Option<Vec<CompactString>>>,
}

impl<'a> ComponentUses<'a> {
    /// `None` when some block that shares the module scope cannot be read.
    pub(super) fn new(
        built: BlockSemantic<'a, 'a>,
        sfc: SfcScriptContext<'a>,
        emit_decorator_metadata: bool,
    ) -> Option<Self> {
        let own = Block::new(built, emit_decorator_metadata)?;
        let has_sibling = sfc.is_sfc && !sfc.sole_script_block;
        let sibling = if has_sibling {
            let program: &'a Program<'a> = sfc.sibling_block()?;
            let built = BlockSemantic::of(sfc.semantic_of(program), program, true);
            Some(Block::new(built, emit_decorator_metadata)?)
        } else {
            None
        };
        // The template resolves names in the setup scope, which holds the
        // imports of both blocks. A component without `<script setup>` hands
        // the template only what its options return.
        let setup = sfc.is_script_setup || has_sibling;
        let template = if sfc.is_sfc && setup {
            sfc.template_side
        } else {
            TemplateSideReads::Nothing
        };
        if !template.is_complete() {
            return None;
        }
        Some(Self {
            own,
            sibling,
            template,
            generic: sfc.setup_generic.filter(|_| setup),
            generic_types: None,
        })
    }

    /// Uses of the import binding `symbol`, spelled `name`. Only a binding
    /// of the `shared` module scope is visible to the other blocks.
    pub(super) fn of(&mut self, symbol: SymbolId, name: &str, shared: bool) -> Uses {
        let mut uses = Uses::default();
        self.own.symbol_uses(symbol, &mut uses);
        if !shared {
            return uses;
        }
        if let Some(sibling) = &self.sibling {
            sibling.unresolved_uses(name, &mut uses);
        }
        if !uses.value && self.template.may_read(name) {
            uses.value = true;
        }
        if !uses.value && !uses.ty {
            uses.ty = self.generic_names(name);
        }
        uses
    }

    pub(super) fn semantic(&self) -> &Semantic<'a> {
        self.own.semantic()
    }

    /// The template or style read of `name` that no runtime binding of that
    /// name serves and no other rule reports.
    pub(super) fn unbound_read(&self, name: &str) -> Option<UnboundRead> {
        self.template.unbound_read(name)
    }

    /// Whether `generic="T extends Name"` names the import. An attribute that
    /// does not parse names nothing: the import then stays unused, not typed.
    fn generic_names(&mut self, name: &str) -> bool {
        let Some(generic) = self.generic else {
            return false;
        };
        self.generic_types
            .get_or_insert_with(|| vize_croquis::script_parser::setup_generic_reads(generic))
            .as_ref()
            .is_some_and(|names| names.iter().any(|read| read == name))
    }
}
