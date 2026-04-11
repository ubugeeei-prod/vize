//! The main `ReactivityTracker` implementation.

use vize_carton::{cstr, CompactString, FxHashMap, FxHashSet, SmallVec};

use super::{
    BindingState, ReactiveBinding, ReactiveBindingId, ReactiveOrigin, ReactiveScope,
    ReactivityViolation, UseSite, UseSiteKind, ViolationKind, ViolationSeverity,
};

/// Ultra-strict reactivity tracker for Vue's reactive system.
pub struct ReactivityTracker {
    /// All tracked bindings.
    pub(crate) bindings: FxHashMap<ReactiveBindingId, ReactiveBinding>,
    /// Bindings by name for lookup.
    pub(crate) bindings_by_name: FxHashMap<CompactString, SmallVec<[ReactiveBindingId; 2]>>,
    /// Scope stack.
    pub(crate) scopes: Vec<ReactiveScope>,
    /// Current scope depth.
    pub(crate) current_scope: u32,
    /// Detected violations.
    pub(crate) violations: Vec<ReactivityViolation>,
    /// Next binding ID.
    pub(crate) next_id: u32,
    /// Whether we're inside a setup function.
    pub(crate) in_setup: bool,
    /// Whether we're inside a template.
    pub(crate) in_template: bool,
}

impl Default for ReactivityTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl ReactivityTracker {
    /// Create a new tracker.
    pub fn new() -> Self {
        Self {
            bindings: FxHashMap::default(),
            bindings_by_name: FxHashMap::default(),
            scopes: vec![ReactiveScope {
                depth: 0,
                bindings: FxHashSet::default(),
                is_setup_scope: false,
                is_async: false,
                parent_scope: None,
            }],
            current_scope: 0,
            violations: Vec::new(),
            next_id: 0,
            in_setup: false,
            in_template: false,
        }
    }

    /// Enter setup scope.
    pub fn enter_setup(&mut self) {
        self.in_setup = true;
        self.push_scope(true, false);
    }

    /// Exit setup scope.
    pub fn exit_setup(&mut self) {
        self.in_setup = false;
        self.pop_scope();
    }

    /// Enter template context.
    pub fn enter_template(&mut self) {
        self.in_template = true;
    }

    /// Exit template context.
    pub fn exit_template(&mut self) {
        self.in_template = false;
    }

    /// Push a new scope.
    pub fn push_scope(&mut self, is_setup_scope: bool, is_async: bool) {
        let new_depth = self.current_scope + 1;
        self.scopes.push(ReactiveScope {
            depth: new_depth,
            bindings: FxHashSet::default(),
            is_setup_scope,
            is_async,
            parent_scope: Some(self.current_scope),
        });
        self.current_scope = new_depth;
    }

    /// Pop current scope.
    pub fn pop_scope(&mut self) {
        if self.current_scope > 0 {
            self.scopes.pop();
            self.current_scope -= 1;
        }
    }

    /// Register a new reactive binding.
    pub fn add_binding(
        &mut self,
        name: CompactString,
        origin: ReactiveOrigin,
        is_mutable: bool,
        start: u32,
        end: u32,
    ) -> ReactiveBindingId {
        let id = ReactiveBindingId::new(self.next_id);
        self.next_id += 1;

        let binding = ReactiveBinding {
            id,
            name: name.clone(),
            origin,
            state: BindingState::Active,
            is_mutable,
            start,
            end,
            scope_depth: self.current_scope,
            value_accessed: false,
            derived_bindings: SmallVec::new(),
            use_sites: SmallVec::new(),
        };

        self.bindings.insert(id, binding);
        self.bindings_by_name.entry(name).or_default().push(id);

        if let Some(scope) = self.scopes.get_mut(self.current_scope as usize) {
            scope.bindings.insert(id);
        }

        id
    }

    /// Look up a binding by name in current scope chain.
    pub fn lookup_binding(&self, name: &str) -> Option<ReactiveBindingId> {
        self.bindings_by_name
            .get(name)
            .and_then(|ids| ids.last().copied())
    }

    /// Record a use of a binding.
    pub fn record_use(
        &mut self,
        binding_id: ReactiveBindingId,
        kind: UseSiteKind,
        start: u32,
        end: u32,
    ) {
        if let Some(binding) = self.bindings.get_mut(&binding_id) {
            // Track .value access
            if matches!(kind, UseSiteKind::ValueAccess) {
                binding.value_accessed = true;
            }

            binding.use_sites.push(UseSite {
                kind: kind.clone(),
                start,
                end,
            });

            // Check for violations based on use kind
            self.check_use_violations(binding_id, &kind, start, end);
        }
    }

    /// Check for violations based on how a binding is used.
    fn check_use_violations(
        &mut self,
        binding_id: ReactiveBindingId,
        kind: &UseSiteKind,
        start: u32,
        end: u32,
    ) {
        let binding = match self.bindings.get(&binding_id) {
            Some(b) => b.clone(),
            None => return,
        };

        match kind {
            UseSiteKind::Destructure { extracted_props } => {
                if binding.loses_reactivity_on_destructure() {
                    let (violation_kind, suggestion) = match &binding.origin {
                        ReactiveOrigin::PiniaStore => (
                            ViolationKind::PiniaDestructure,
                            Some(CompactString::new(
                                "Use storeToRefs() for reactive state/getters",
                            )),
                        ),
                        ReactiveOrigin::Props => (
                            ViolationKind::PropsDestructure,
                            Some(CompactString::new(
                                "Use toRefs(props) or toRef(props, 'propName')",
                            )),
                        ),
                        ReactiveOrigin::Inject => (
                            ViolationKind::InjectDestructure,
                            Some(CompactString::new(
                                "Access injected properties directly without destructuring",
                            )),
                        ),
                        _ => (
                            ViolationKind::DestructuringLoss {
                                extracted_props: extracted_props.clone(),
                            },
                            Some(CompactString::new(
                                "Use toRefs() to maintain reactivity, or access properties directly",
                            )),
                        ),
                    };

                    self.violations.push(ReactivityViolation {
                        binding_id,
                        kind: violation_kind,
                        start,
                        end,
                        message: cstr!(
                            "Destructuring '{}' loses reactivity for: {}",
                            binding.name,
                            extracted_props.join(", ")
                        ),
                        suggestion,
                        severity: ViolationSeverity::Error,
                    });
                }
            }

            UseSiteKind::Spread => {
                if binding.loses_reactivity_on_spread() {
                    self.violations.push(ReactivityViolation {
                        binding_id,
                        kind: ViolationKind::SpreadLoss,
                        start,
                        end,
                        message: cstr!(
                            "Spreading '{}' creates a non-reactive copy",
                            binding.name
                        ),
                        suggestion: Some(CompactString::new(
                            "Use Object.assign() to merge into reactive object, or toRaw() for intentional copy",
                        )),
                        severity: ViolationSeverity::Error,
                    });
                }
            }

            UseSiteKind::Reassignment => {
                if !binding.is_mutable {
                    self.violations.push(ReactivityViolation {
                        binding_id,
                        kind: ViolationKind::ReactiveConst,
                        start,
                        end,
                        message: cstr!(
                            "Cannot reassign '{}' declared with const",
                            binding.name
                        ),
                        suggestion: Some(CompactString::new(
                            "Use let instead of const if reassignment is needed, or mutate the object's properties",
                        )),
                        severity: ViolationSeverity::Error,
                    });
                } else if binding.origin.reactive_type().is_reactive() {
                    // Warn about losing reactive tracking
                    self.violations.push(ReactivityViolation {
                        binding_id,
                        kind: ViolationKind::Reassignment,
                        start,
                        end,
                        message: cstr!(
                            "Reassigning '{}' breaks reactivity tracking",
                            binding.name
                        ),
                        suggestion: Some(CompactString::new(
                            "Mutate the object's properties instead, or use ref() for replaceable values",
                        )),
                        severity: ViolationSeverity::Warning,
                    });
                }
            }

            UseSiteKind::ExternalEscape { target } => {
                self.violations.push(ReactivityViolation {
                    binding_id,
                    kind: ViolationKind::ExternalMutation,
                    start,
                    end,
                    message: cstr!(
                        "Reactive object '{}' assigned to external target '{target}' - external code may mutate state",
                        binding.name
                    ),
                    suggestion: Some(CompactString::new(
                        "Use toRaw() or structuredClone(toRaw()) to pass non-reactive copy",
                    )),
                    severity: ViolationSeverity::Warning,
                });
            }

            UseSiteKind::ClosureCapture { closure_start: _ } => {
                // Check if this closure might escape (e.g., setTimeout, addEventListener)
                // For now, we'll warn about all captures in potentially escaping closures
                self.violations.push(ReactivityViolation {
                    binding_id,
                    kind: ViolationKind::UnsafeClosureCapture,
                    start,
                    end,
                    message: cstr!(
                        "Reactive reference '{}' captured in closure",
                        binding.name
                    ),
                    suggestion: Some(CompactString::new(
                        "Ensure closure doesn't outlive component, or use watchEffect for reactive effects",
                    )),
                    severity: ViolationSeverity::Info,
                });
            }

            UseSiteKind::Read if binding.is_ref_type() && !self.in_template => {
                // Ref used without .value outside template
                // This might be intentional (passing to function that handles refs)
                // so we only emit a hint
                if !binding.value_accessed {
                    self.violations.push(ReactivityViolation {
                        binding_id,
                        kind: ViolationKind::MissingValueAccess,
                        start,
                        end,
                        message: cstr!(
                            "Ref '{}' used without .value - did you mean {}.value?",
                            binding.name, binding.name
                        ),
                        suggestion: Some(CompactString::new(
                            "Access .value to get/set the underlying value, or use unref() for conditional unwrapping",
                        )),
                        severity: ViolationSeverity::Hint,
                    });
                }
            }

            _ => {}
        }
    }

    /// Mark a binding as having lost reactivity.
    pub fn mark_reactivity_lost(&mut self, binding_id: ReactiveBindingId) {
        if let Some(binding) = self.bindings.get_mut(&binding_id) {
            binding.state = BindingState::ReactivityLost;
        }
    }

    /// Mark a binding as escaped.
    pub fn mark_escaped(&mut self, binding_id: ReactiveBindingId) {
        if let Some(binding) = self.bindings.get_mut(&binding_id) {
            binding.state = BindingState::Escaped;
        }
    }

    /// Get all violations.
    pub fn violations(&self) -> &[ReactivityViolation] {
        &self.violations
    }

    /// Get all bindings.
    pub fn bindings(&self) -> impl Iterator<Item = &ReactiveBinding> {
        self.bindings.values()
    }

    /// Get a specific binding.
    pub fn get_binding(&self, id: ReactiveBindingId) -> Option<&ReactiveBinding> {
        self.bindings.get(&id)
    }
}

#[cfg(test)]
mod tests {
    use super::{CompactString, ReactiveOrigin, ReactivityTracker, UseSiteKind, ViolationKind};

    #[test]
    fn test_basic_tracking() {
        let mut tracker = ReactivityTracker::new();
        tracker.enter_setup();

        let _id = tracker.add_binding(
            CompactString::new("state"),
            ReactiveOrigin::Reactive,
            false,
            0,
            10,
        );

        assert!(tracker.lookup_binding("state").is_some());
        assert_eq!(tracker.bindings().count(), 1);
    }

    #[test]
    fn test_destructuring_violation() {
        let mut tracker = ReactivityTracker::new();
        tracker.enter_setup();

        let id = tracker.add_binding(
            CompactString::new("state"),
            ReactiveOrigin::Reactive,
            false,
            0,
            10,
        );

        tracker.record_use(
            id,
            UseSiteKind::Destructure {
                extracted_props: vec![CompactString::new("a"), CompactString::new("b")],
            },
            20,
            40,
        );

        assert_eq!(tracker.violations().len(), 1);
        assert!(matches!(
            tracker.violations()[0].kind,
            ViolationKind::DestructuringLoss { .. }
        ));
    }

    #[test]
    fn test_spread_violation() {
        let mut tracker = ReactivityTracker::new();
        tracker.enter_setup();

        let id = tracker.add_binding(
            CompactString::new("state"),
            ReactiveOrigin::Reactive,
            false,
            0,
            10,
        );

        tracker.record_use(id, UseSiteKind::Spread, 20, 30);

        assert_eq!(tracker.violations().len(), 1);
        assert!(matches!(
            tracker.violations()[0].kind,
            ViolationKind::SpreadLoss
        ));
    }

    #[test]
    fn test_pinia_destructure() {
        let mut tracker = ReactivityTracker::new();
        tracker.enter_setup();

        let id = tracker.add_binding(
            CompactString::new("store"),
            ReactiveOrigin::PiniaStore,
            false,
            0,
            10,
        );

        tracker.record_use(
            id,
            UseSiteKind::Destructure {
                extracted_props: vec![CompactString::new("count")],
            },
            20,
            40,
        );

        assert_eq!(tracker.violations().len(), 1);
        assert!(matches!(
            tracker.violations()[0].kind,
            ViolationKind::PiniaDestructure
        ));
    }

    #[test]
    fn test_ref_without_value() {
        let mut tracker = ReactivityTracker::new();
        tracker.enter_setup();

        let id = tracker.add_binding(
            CompactString::new("count"),
            ReactiveOrigin::Ref,
            false,
            0,
            10,
        );

        // Use ref without .value outside template
        tracker.record_use(id, UseSiteKind::Read, 20, 25);

        assert_eq!(tracker.violations().len(), 1);
        assert!(matches!(
            tracker.violations()[0].kind,
            ViolationKind::MissingValueAccess
        ));
    }

    #[test]
    fn test_ref_in_template() {
        let mut tracker = ReactivityTracker::new();
        tracker.enter_setup();

        let id = tracker.add_binding(
            CompactString::new("count"),
            ReactiveOrigin::Ref,
            false,
            0,
            10,
        );

        // Use ref in template (auto-unwrap is OK)
        tracker.enter_template();
        tracker.record_use(id, UseSiteKind::Read, 20, 25);
        tracker.exit_template();

        // No violation in template context
        assert!(tracker.violations().is_empty());
    }

    #[test]
    fn test_markdown_report() {
        let mut tracker = ReactivityTracker::new();
        tracker.enter_setup();

        tracker.add_binding(
            CompactString::new("state"),
            ReactiveOrigin::Reactive,
            false,
            0,
            10,
        );

        let md = tracker.to_markdown();
        insta::assert_snapshot!(md.as_str());
    }
}
