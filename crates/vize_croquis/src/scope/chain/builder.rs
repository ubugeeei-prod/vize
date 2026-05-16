//! Scope creation and entry methods for [`ScopeChain`].
//!
//! Provides all `enter_*` methods that create new scopes and push them
//! onto the chain (v-for, v-slot, event handler, callback, module, etc.).

use super::{
    smallvec, BindingType, BlockScopeData, CallbackScopeData, ClientOnlyScopeData,
    ClosureScopeData, CompactString, EventHandlerScopeData, ExternalModuleScopeData,
    JsGlobalScopeData, NonScriptSetupScopeData, ParentScopes, Scope, ScopeBinding, ScopeChain,
    ScopeData, ScopeId, ScopeKind, ScriptSetupScopeData, UniversalScopeData, VForScopeData,
    VSlotScopeData, VueGlobalScopeData,
};

impl ScopeChain {
    /// Enter a new scope
    #[inline]
    pub fn enter_scope(&mut self, kind: ScopeKind) -> ScopeId {
        let id = ScopeId::new(self.scopes.len() as u32);
        let scope = Scope::new(id, Some(self.current), kind);
        self.scopes.push(scope);
        self.current = id;
        id
    }

    /// Enter a new scope with Vue global access (for template scopes)
    #[inline]
    pub fn enter_scope_with_vue_global(&mut self, kind: ScopeKind) -> ScopeId {
        let id = ScopeId::new(self.scopes.len() as u32);
        let mut parents: ParentScopes = smallvec![self.current];

        // Add Vue global scope as additional parent if it exists
        if let Some(vue_id) = self.find_scope_by_kind(ScopeKind::VueGlobal) {
            if !parents.contains(&vue_id) {
                parents.push(vue_id);
            }
        }

        let scope = Scope::with_parents(id, parents, kind);
        self.scopes.push(scope);
        self.current = id;
        id
    }

    /// Exit the current scope and return to primary parent
    #[inline]
    pub fn exit_scope(&mut self) {
        if let Some(parent) = self.current_scope().parent() {
            self.current = parent;
        }
    }

    /// Enter a v-for scope with the given data
    pub fn enter_v_for_scope(&mut self, data: VForScopeData, start: u32, end: u32) -> ScopeId {
        let id = ScopeId::new(self.scopes.len() as u32);
        let parents = self.build_template_parents();
        let mut scope = Scope::with_span_parents(id, parents, ScopeKind::VFor, start, end);

        // Add value alias as binding
        scope.add_binding(
            data.value_alias.clone(),
            ScopeBinding::new(BindingType::SetupConst, start),
        );

        // Add key alias if present
        if let Some(ref key) = data.key_alias {
            scope.add_binding(
                key.clone(),
                ScopeBinding::new(BindingType::SetupConst, start),
            );
        }

        // Add index alias if present
        if let Some(ref index) = data.index_alias {
            scope.add_binding(
                index.clone(),
                ScopeBinding::new(BindingType::SetupConst, start),
            );
        }

        scope.set_data(ScopeData::VFor(data));
        self.scopes.push(scope);
        self.current = id;
        id
    }

    /// Enter a v-slot scope with the given data
    pub fn enter_v_slot_scope(&mut self, data: VSlotScopeData, start: u32, end: u32) -> ScopeId {
        let id = ScopeId::new(self.scopes.len() as u32);
        let parents = self.build_template_parents();
        let mut scope = Scope::with_span_parents(id, parents, ScopeKind::VSlot, start, end);

        // Add prop names as bindings
        for prop_name in &data.prop_names {
            scope.add_binding(
                prop_name.clone(),
                ScopeBinding::new(BindingType::SetupConst, start),
            );
        }

        scope.set_data(ScopeData::VSlot(data));
        self.scopes.push(scope);
        self.current = id;
        id
    }

    /// Enter an event handler scope
    pub fn enter_event_handler_scope(
        &mut self,
        data: EventHandlerScopeData,
        start: u32,
        end: u32,
    ) -> ScopeId {
        let id = ScopeId::new(self.scopes.len() as u32);
        let parents = self.build_template_parents();
        let mut scope = Scope::with_span_parents(id, parents, ScopeKind::EventHandler, start, end);

        // Add implicit $event binding if applicable
        if data.has_implicit_event {
            scope.add_binding(
                CompactString::const_new("$event"),
                ScopeBinding::new(BindingType::SetupConst, start),
            );
        }

        // Add explicit parameter names as bindings
        for param_name in &data.param_names {
            scope.add_binding(
                param_name.clone(),
                ScopeBinding::new(BindingType::SetupConst, start),
            );
        }

        scope.set_data(ScopeData::EventHandler(data));
        self.scopes.push(scope);
        self.current = id;
        id
    }

    /// Enter a callback/arrow function scope (script context - no vue global)
    pub fn enter_callback_scope(
        &mut self,
        data: CallbackScopeData,
        start: u32,
        end: u32,
    ) -> ScopeId {
        let id = ScopeId::new(self.scopes.len() as u32);
        // Script callbacks only have current scope as parent (no vue global)
        let mut scope = Scope::with_span(id, Some(self.current), ScopeKind::Callback, start, end);

        // Add parameter names as bindings
        for param_name in &data.param_names {
            scope.add_binding(
                param_name.clone(),
                ScopeBinding::new(BindingType::SetupConst, start),
            );
        }

        scope.set_data(ScopeData::Callback(data));
        self.scopes.push(scope);
        self.current = id;
        id
    }

    /// Enter a callback scope with vue global access (for template inline expressions)
    pub fn enter_template_callback_scope(
        &mut self,
        data: CallbackScopeData,
        start: u32,
        end: u32,
    ) -> ScopeId {
        let id = ScopeId::new(self.scopes.len() as u32);
        let parents = self.build_template_parents();
        let mut scope = Scope::with_span_parents(id, parents, ScopeKind::Callback, start, end);

        // Add parameter names as bindings
        for param_name in &data.param_names {
            scope.add_binding(
                param_name.clone(),
                ScopeBinding::new(BindingType::SetupConst, start),
            );
        }

        scope.set_data(ScopeData::Callback(data));
        self.scopes.push(scope);
        self.current = id;
        id
    }

    /// Enter a module scope
    pub fn enter_module_scope(&mut self, start: u32, end: u32) -> ScopeId {
        let id = ScopeId::new(self.scopes.len() as u32);
        let scope = Scope::with_span(id, Some(self.current), ScopeKind::Module, start, end);
        self.scopes.push(scope);
        self.current = id;
        id
    }

    /// Enter a script setup scope
    pub fn enter_script_setup_scope(
        &mut self,
        data: ScriptSetupScopeData,
        start: u32,
        end: u32,
    ) -> ScopeId {
        let id = ScopeId::new(self.scopes.len() as u32);
        let mut scope =
            Scope::with_span(id, Some(self.current), ScopeKind::ScriptSetup, start, end);
        scope.set_data(ScopeData::ScriptSetup(data));
        self.scopes.push(scope);
        self.current = id;
        id
    }

    /// Enter a non-script-setup scope (Options API, regular script)
    pub fn enter_non_script_setup_scope(
        &mut self,
        data: NonScriptSetupScopeData,
        start: u32,
        end: u32,
    ) -> ScopeId {
        let id = ScopeId::new(self.scopes.len() as u32);
        let mut scope = Scope::with_span(
            id,
            Some(self.current),
            ScopeKind::NonScriptSetup,
            start,
            end,
        );
        scope.set_data(ScopeData::NonScriptSetup(data));
        self.scopes.push(scope);
        self.current = id;
        id
    }

    /// Enter a universal scope (SSR - runs on both server and client)
    pub fn enter_universal_scope(
        &mut self,
        data: UniversalScopeData,
        start: u32,
        end: u32,
    ) -> ScopeId {
        let id = ScopeId::new(self.scopes.len() as u32);
        let mut scope = Scope::with_span(id, Some(self.current), ScopeKind::Universal, start, end);
        scope.set_data(ScopeData::Universal(data));
        self.scopes.push(scope);
        self.current = id;
        id
    }

    /// Enter a client-only scope (onMounted, onBeforeUnmount, etc.)
    /// Parents: current scope + !js (browser globals)
    pub fn enter_client_only_scope(
        &mut self,
        data: ClientOnlyScopeData,
        start: u32,
        end: u32,
    ) -> ScopeId {
        let id = ScopeId::new(self.scopes.len() as u32);

        // Build parents: current scope + !js (browser globals)
        let mut parents: ParentScopes = smallvec![self.current];
        if let Some(browser_id) = self.find_scope_by_kind(ScopeKind::JsGlobalBrowser) {
            if !parents.contains(&browser_id) {
                parents.push(browser_id);
            }
        }

        let mut scope = Scope::with_span_parents(id, parents, ScopeKind::ClientOnly, start, end);
        scope.set_data(ScopeData::ClientOnly(data));
        self.scopes.push(scope);
        self.current = id;
        id
    }

    /// Enter a JavaScript global scope with specific runtime
    pub fn enter_js_global_scope(
        &mut self,
        data: JsGlobalScopeData,
        start: u32,
        end: u32,
    ) -> ScopeId {
        let id = ScopeId::new(self.scopes.len() as u32);
        let scope_kind = data.runtime.to_scope_kind();
        let binding_type = data.runtime.to_binding_type();
        let mut scope = Scope::with_span(id, Some(self.current), scope_kind, start, end);

        // Add JS globals as bindings with runtime-specific type
        for global in &data.globals {
            scope.add_binding(global.clone(), ScopeBinding::new(binding_type, start));
        }

        scope.set_data(ScopeData::JsGlobal(data));
        self.scopes.push(scope);
        self.current = id;
        id
    }

    /// Enter a Vue global scope
    pub fn enter_vue_global_scope(
        &mut self,
        data: VueGlobalScopeData,
        start: u32,
        end: u32,
    ) -> ScopeId {
        let id = ScopeId::new(self.scopes.len() as u32);
        let mut scope = Scope::with_span(id, Some(self.current), ScopeKind::VueGlobal, start, end);

        // Add Vue globals as bindings
        for global in &data.globals {
            scope.add_binding(
                global.clone(),
                ScopeBinding::new(BindingType::VueGlobal, start),
            );
        }

        scope.set_data(ScopeData::VueGlobal(data));
        self.scopes.push(scope);
        self.current = id;
        id
    }

    /// Enter an external module scope
    pub fn enter_external_module_scope(
        &mut self,
        data: ExternalModuleScopeData,
        start: u32,
        end: u32,
    ) -> ScopeId {
        let id = ScopeId::new(self.scopes.len() as u32);
        let mut scope = Scope::with_span(
            id,
            Some(self.current),
            ScopeKind::ExternalModule,
            start,
            end,
        );
        scope.set_data(ScopeData::ExternalModule(data));
        self.scopes.push(scope);
        self.current = id;
        id
    }

    /// Enter a closure scope (function declaration, function expression, arrow function)
    pub fn enter_closure_scope(&mut self, data: ClosureScopeData, start: u32, end: u32) -> ScopeId {
        let id = ScopeId::new(self.scopes.len() as u32);
        let mut scope = Scope::with_span(id, Some(self.current), ScopeKind::Closure, start, end);

        // Add parameter names as bindings
        for param in &data.param_names {
            scope.add_binding(
                param.clone(),
                ScopeBinding::new(BindingType::SetupConst, start),
            );
        }

        scope.set_data(ScopeData::Closure(data));
        self.scopes.push(scope);
        self.current = id;
        id
    }

    /// Enter a block scope (if, for, switch, try, catch, etc.)
    pub fn enter_block_scope(&mut self, data: BlockScopeData, start: u32, end: u32) -> ScopeId {
        let id = ScopeId::new(self.scopes.len() as u32);
        let mut scope = Scope::with_span(id, Some(self.current), ScopeKind::Block, start, end);
        scope.set_data(ScopeData::Block(data));
        self.scopes.push(scope);
        self.current = id;
        id
    }
}
