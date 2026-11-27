//! Which instance runs a `useTemplateRef` call.
//!
//! The setup body runs in the component's own instance, and so does a callback
//! it hands straight to a call (`onMounted(() => ...)`), the `setup` of the
//! default-exported component, and a local function it calls, directly or
//! through other local functions. A local function the
//! setup never calls can run in another instance: a function rendered as a
//! component (`<Comp />`), or a value passed elsewhere. Its calls keep the
//! inferred signature, but a key missing from this template is not an error
//! there.

use oxc_ast::ast::{
    Argument, BindingPattern, CallExpression, Expression, ObjectProperty, VariableDeclarator,
};
use oxc_semantic::Scoping;
use oxc_syntax::symbol::SymbolId;
use vize_carton::FxHashSet;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Owner {
    /// The setup body or a callback it passes straight to a call.
    Setup,
    /// The local function declared by this binding.
    Local(SymbolId),
    /// A function no binding names, such as an object method.
    Detached,
}

impl Owner {
    /// Whether this component's instance runs the call, given the local
    /// functions setup reaches.
    pub(super) fn runs_in_instance(self, reached: &FxHashSet<SymbolId>) -> bool {
        match self {
            Self::Setup => true,
            Self::Local(function) => reached.contains(&function),
            Self::Detached => false,
        }
    }
}

pub(super) struct Owners {
    current: Owner,
    stack: Vec<Owner>,
    /// Starts of functions that run as setup does: callbacks setup passes
    /// straight to a call, and the default export's `setup`.
    instance_functions: FxHashSet<u32>,
    /// A function that initializes a setup binding, and that binding.
    named: Option<(u32, SymbolId)>,
    /// Local functions called from each owner.
    edges: Vec<(Owner, SymbolId)>,
}

impl Default for Owners {
    fn default() -> Self {
        Self {
            current: Owner::Setup,
            stack: Vec::new(),
            instance_functions: FxHashSet::default(),
            named: None,
            edges: Vec::new(),
        }
    }
}

impl Owners {
    pub(super) fn current(&self) -> Owner {
        self.current
    }

    /// Enter the function starting at `start`; `id` is its declared name.
    pub(super) fn enter(&mut self, start: u32, id: Option<SymbolId>) {
        let owner = if self.current != Owner::Setup {
            self.current
        } else if self.instance_functions.contains(&start) {
            Owner::Setup
        } else if let Some(symbol) = id.or_else(|| {
            self.named
                .filter(|(at, _)| *at == start)
                .map(|(_, symbol)| symbol)
        }) {
            Owner::Local(symbol)
        } else {
            Owner::Detached
        };
        self.stack.push(self.current);
        self.current = owner;
    }

    pub(super) fn leave(&mut self) {
        self.current = self.stack.pop().unwrap_or(Owner::Setup);
    }

    /// Remember `const name = () => ...` so the function is owned by `name`.
    pub(super) fn name_initializer(&mut self, declarator: &VariableDeclarator<'_>) {
        if self.current != Owner::Setup {
            return;
        }
        let BindingPattern::BindingIdentifier(id) = &declarator.id else {
            return;
        };
        let start = match &declarator.init {
            Some(Expression::ArrowFunctionExpression(function)) => function.span.start,
            Some(Expression::FunctionExpression(function)) => function.span.start,
            _ => return,
        };
        if let Some(symbol) = id.symbol_id.get() {
            self.named = Some((start, symbol));
        }
    }

    /// `setup` of the default-exported component runs in this instance.
    pub(super) fn observe_default_export_property(&mut self, property: &ObjectProperty<'_>) {
        if self.current != Owner::Setup || property.key.static_name().as_deref() != Some("setup") {
            return;
        }
        let start = match &property.value {
            Expression::ArrowFunctionExpression(function) => function.span.start,
            Expression::FunctionExpression(function) => function.span.start,
            _ => return,
        };
        self.instance_functions.insert(start);
    }

    /// Record the local function a call invokes, and the callbacks setup
    /// passes to it.
    pub(super) fn observe_call(&mut self, call: &CallExpression<'_>, scoping: &Scoping) {
        if let Expression::Identifier(callee) = &call.callee
            && let Some(symbol) = callee
                .reference_id
                .get()
                .and_then(|reference| scoping.get_reference(reference).symbol_id())
        {
            self.edges.push((self.current, symbol));
        }
        if self.current != Owner::Setup {
            return;
        }
        for argument in &call.arguments {
            match argument {
                Argument::ArrowFunctionExpression(function) => {
                    self.instance_functions.insert(function.span.start);
                }
                Argument::FunctionExpression(function) => {
                    self.instance_functions.insert(function.span.start);
                }
                _ => {}
            }
        }
    }

    /// The local functions that run in this component's instance.
    pub(super) fn reached_from_setup(&self) -> FxHashSet<SymbolId> {
        let mut reached = FxHashSet::default();
        loop {
            let before = reached.len();
            for (owner, callee) in &self.edges {
                if owner.runs_in_instance(&reached) {
                    reached.insert(*callee);
                }
            }
            if reached.len() == before {
                return reached;
            }
        }
    }
}
