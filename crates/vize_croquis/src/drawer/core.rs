use crate::croquis::Croquis;
use vize_carton::{CompactString, FxHashMap, profile};

use super::DrawerOptions;

/// High-performance Vue SFC drawer.
///
/// Uses lazy evaluation and efficient data structures to minimize overhead.
pub struct Drawer {
    pub(crate) options: DrawerOptions,
    /// Unused binding population is an explicit lint demand.
    pub(crate) track_unused_bindings: bool,
    /// Resolve Vue 3 Options API template bindings (opt-in, standard build).
    pub(crate) options_api: bool,
    /// Legacy Vue 2.7 / Nuxt 2: implies `options_api` plus Nuxt 2 globals.
    pub(crate) legacy_vue2: bool,
    pub(crate) croquis: Croquis,
    /// Track if script was analyzed (for undefined detection)
    pub(crate) script_drawn: bool,
    /// Current v-if guard stack (for type narrowing in templates)
    pub(crate) vif_guard_stack: Vec<CompactString>,
    /// Memoized join of `vif_guard_stack` (` && `-separated). `None` when the
    /// stack is empty. Recomputed eagerly whenever `vif_guard_stack` is pushed
    /// to or popped (both happen behind `&mut self`), so the read path
    /// (`current_vif_guard`) is a cheap `&self` clone. A plain `Option` (rather
    /// than interior mutability) keeps `Drawer: Sync`.
    pub(crate) vif_guard_cache: Option<CompactString>,
    /// Conditions of the preceding `v-if` / `v-else-if` siblings in the current
    /// sibling group. Used to build the negated guard for a flat `v-else` /
    /// `v-else-if` element when the parser keeps branches as sibling elements
    /// (rather than grouping them into an `IfNode`).
    pub(crate) vif_branch_conditions: Vec<CompactString>,
    /// Number of v-for scopes currently entered. `is_in_vfor_scope` reads this
    /// instead of walking the parent scope chain. Incremented on v-for scope
    /// enter, decremented on exit (paired with `vif_guard_stack` discipline).
    pub(crate) vfor_depth: u32,
    /// Component tag stack for the current element ancestry. Used by
    /// `<template #name>` slot hosts to recover the owning child component.
    pub(crate) parent_component_stack: Vec<CompactString>,
    /// The template source string node-loc spans index into. Captured from
    /// `RootNode::source` when `draw_template` starts so the walk can recover
    /// covered text from a `SourceLocation` span.
    pub(crate) template_source: CompactString,
    /// Memoized identifier extraction keyed by expression text. Template
    /// expressions repeat heavily (e.g. the same `:to`/`@click`/`{{ }}` across
    /// every `v-for` iteration's rendered element), and `extract_identifiers_oxc`
    /// is a pure function of the expression string, so the parse+walk is done
    /// once per distinct expression instead of once per occurrence. The cached
    /// `Vec` is read by reference (disjoint field borrow), so cache hits avoid
    /// both the parse and any clone.
    pub(crate) ident_cache: FxHashMap<CompactString, Vec<CompactString>>,
    pub(super) occurrence_capture: Option<Box<super::occurrences::OccurrenceCapture>>,
}

impl Drawer {
    pub(crate) fn checks_binding_reads(&self) -> bool {
        self.options.detect_undefined || self.track_unused_bindings
    }

    /// Create a new drawer with default options
    #[inline]
    pub fn new() -> Self {
        Self::with_options(DrawerOptions::default())
    }

    /// Create drawer with specific options
    #[inline]
    pub fn with_options(options: DrawerOptions) -> Self {
        let drawer = Self {
            options,
            track_unused_bindings: false,
            options_api: false,
            legacy_vue2: false,
            croquis: Croquis::new(),
            script_drawn: false,
            vif_guard_stack: Vec::new(),
            vif_guard_cache: None,
            vif_branch_conditions: Vec::new(),
            vfor_depth: 0,
            parent_component_stack: Vec::new(),
            template_source: CompactString::default(),
            ident_cache: FxHashMap::default(),
            occurrence_capture: None,
        };
        if options.analyze_script
            && options.detect_undefined
            && options.collect_template_expressions
        {
            drawer.with_binding_occurrences()
        } else {
            drawer
        }
    }

    /// Continue drawing from an existing croquis.
    ///
    /// This is useful for infrastructure that needs to normalize script offsets
    /// before adding template facts to the same Croquis.
    #[inline]
    pub fn with_croquis(options: DrawerOptions, mut croquis: Croquis, script_drawn: bool) -> Self {
        let packet = croquis.setup_context.take_ssr_occurrences();
        let drawer = Self {
            options,
            track_unused_bindings: false,
            options_api: false,
            legacy_vue2: false,
            croquis,
            script_drawn,
            vif_guard_stack: Vec::new(),
            vif_guard_cache: None,
            vif_branch_conditions: Vec::new(),
            vfor_depth: 0,
            parent_component_stack: Vec::new(),
            template_source: CompactString::default(),
            ident_cache: FxHashMap::default(),
            occurrence_capture: None,
        };
        match packet {
            Some(packet) => drawer.with_binding_occurrence_packet(packet),
            None if options.analyze_script
                && options.detect_undefined
                && options.collect_template_expressions =>
            {
                drawer.with_binding_occurrences()
            }
            None => drawer,
        }
    }

    /// Collect unread setup bindings using the already parsed script AST.
    pub fn with_unused_bindings(mut self) -> Self {
        self.track_unused_bindings = true;
        self
    }

    /// Compatibility wrapper for the old Analyzer naming.
    #[inline]
    pub fn with_summary(options: DrawerOptions, croquis: Croquis, script_drawn: bool) -> Self {
        Self::with_croquis(options, croquis, script_drawn)
    }

    /// Resolve Vue 3 Options API template bindings (opt-in, standard build).
    #[inline]
    pub fn with_options_api(mut self) -> Self {
        self.options_api = true;
        self
    }

    /// Enable Vue 2.7 / Nuxt 2 compatibility helpers (implies Options API).
    #[inline]
    pub fn with_legacy_vue2(mut self) -> Self {
        self.legacy_vue2 = true;
        self
    }

    /// Get the current v-if guard (combined from stack).
    ///
    /// The joined string is invariant for a given `vif_guard_stack` state, so it
    /// is memoized in `vif_guard_cache` (recomputed by `refresh_vif_guard_cache`
    /// on every push/pop) and this read path is a cheap clone.
    pub(crate) fn current_vif_guard(&self) -> Option<CompactString> {
        self.vif_guard_cache.clone()
    }

    /// Recompute the memoized joined v-if guard. Call after every push/pop of
    /// `vif_guard_stack` (both behind `&mut self`) to keep the cache current.
    pub(crate) fn refresh_vif_guard_cache(&mut self) {
        self.vif_guard_cache = if self.vif_guard_stack.is_empty() {
            None
        } else {
            Some(CompactString::new(self.vif_guard_stack.join(" && ")))
        };
    }

    /// Create drawer for linting (optimized)
    #[inline]
    pub fn for_lint() -> Self {
        Self::with_options(DrawerOptions::for_lint())
    }

    /// Create drawer for compilation
    #[inline]
    pub fn for_compile() -> Self {
        Self::with_options(DrawerOptions::compile_demand())
    }

    /// Finish drawing and return the croquis.
    ///
    /// Consumes the drawer.
    #[inline]
    pub fn finish(self) -> Croquis {
        let demanded = self.occurrence_capture.is_some();
        profile!("croquis.drawer.finish", {
            let (mut croquis, packet) = self.finish_occurrence_packet(false, false);
            // Keep complete ownership through split-script and template joins,
            // including an import-only block needed by its sibling.
            if demanded {
                croquis.setup_context.set_ssr_occurrences(packet);
            }
            croquis
        })
    }

    /// Get a reference to the current croquis (without consuming).
    #[inline]
    pub fn croquis(&self) -> &Croquis {
        &self.croquis
    }

    /// Compatibility wrapper for the old Analyzer naming.
    #[inline]
    pub fn summary(&self) -> &Croquis {
        self.croquis()
    }

    /// Get a mutable reference to the current croquis (croquis).
    ///
    /// This is primarily used for testing and advanced scenarios where
    /// the caller needs to inject data (e.g., used_components from template parsing).
    #[inline]
    pub fn croquis_mut(&mut self) -> &mut Croquis {
        &mut self.croquis
    }
}

impl Default for Drawer {
    fn default() -> Self {
        Self::new()
    }
}
