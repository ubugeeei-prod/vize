/// Scope kind for display
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeKind {
    Module,
    Function,
    Block,
    VFor,
    VSlot,
    EventHandler,
    Callback,
    ScriptSetup,
    NonScriptSetup,
    Universal,
    ClientOnly,
    JsGlobalUniversal,
    JsGlobalBrowser,
    JsGlobalNode,
    JsGlobalDeno,
    JsGlobalBun,
    VueGlobal,
    ExternalModule,
    Closure,
    VMatch,
    VWhen,
}

impl From<crate::scope::ScopeKind> for ScopeKind {
    fn from(kind: crate::scope::ScopeKind) -> Self {
        match kind {
            crate::scope::ScopeKind::Module => Self::Module,
            crate::scope::ScopeKind::Function => Self::Function,
            crate::scope::ScopeKind::Block => Self::Block,
            crate::scope::ScopeKind::VFor => Self::VFor,
            crate::scope::ScopeKind::VSlot => Self::VSlot,
            crate::scope::ScopeKind::VMatch => Self::VMatch,
            crate::scope::ScopeKind::VWhen => Self::VWhen,
            crate::scope::ScopeKind::EventHandler => Self::EventHandler,
            crate::scope::ScopeKind::Callback => Self::Callback,
            crate::scope::ScopeKind::ScriptSetup => Self::ScriptSetup,
            crate::scope::ScopeKind::NonScriptSetup => Self::NonScriptSetup,
            crate::scope::ScopeKind::Universal => Self::Universal,
            crate::scope::ScopeKind::ClientOnly => Self::ClientOnly,
            crate::scope::ScopeKind::JsGlobalUniversal => Self::JsGlobalUniversal,
            crate::scope::ScopeKind::JsGlobalBrowser => Self::JsGlobalBrowser,
            crate::scope::ScopeKind::JsGlobalNode => Self::JsGlobalNode,
            crate::scope::ScopeKind::JsGlobalDeno => Self::JsGlobalDeno,
            crate::scope::ScopeKind::JsGlobalBun => Self::JsGlobalBun,
            crate::scope::ScopeKind::VueGlobal => Self::VueGlobal,
            crate::scope::ScopeKind::ExternalModule => Self::ExternalModule,
            crate::scope::ScopeKind::Closure => Self::Closure,
        }
    }
}

impl ScopeKind {
    /// Get the display prefix for this scope kind
    ///
    /// - `~` = universal (works on both client and server)
    /// - `!` = client only (requires client API: window, document, etc.)
    /// - `#` = server private (reserved for future Server Components)
    #[inline]
    pub const fn prefix(&self) -> &'static str {
        match self {
            // Client-only (requires client API)
            Self::ClientOnly | Self::JsGlobalBrowser => "!",
            // Server private (reserved for future Server Components)
            Self::JsGlobalNode | Self::JsGlobalDeno | Self::JsGlobalBun => "#",
            // Universal (works on both)
            _ => "~",
        }
    }

    /// Get the display name for this scope kind
    #[inline]
    pub const fn display_name(&self) -> &'static str {
        match self {
            Self::Module => "module",
            Self::Function => "fn",
            Self::Block => "block",
            Self::VFor => "v-for",
            Self::VSlot => "v-slot",
            Self::VMatch => "v-match",
            Self::VWhen => "v-when",
            Self::EventHandler => "event",
            Self::Callback => "callback",
            Self::ScriptSetup => "setup",
            Self::NonScriptSetup => "plain",
            Self::Universal => "universal",
            Self::ClientOnly => "client",
            Self::JsGlobalUniversal => "universal",
            Self::JsGlobalBrowser => "client",
            Self::JsGlobalNode => "server",
            Self::JsGlobalDeno => "server",
            Self::JsGlobalBun => "server",
            Self::VueGlobal => "vue",
            Self::ExternalModule => "extern",
            Self::Closure => "closure",
        }
    }

    /// Format for VIR display (zero allocation)
    #[inline]
    pub const fn to_display(&self) -> &'static str {
        match self {
            Self::Module => "module",
            Self::Function => "fn",
            Self::Block => "block",
            Self::VFor => "v-for",
            Self::VSlot => "v-slot",
            Self::VMatch => "v-match",
            Self::VWhen => "v-when",
            Self::EventHandler => "event",
            Self::Callback => "callback",
            Self::ScriptSetup => "setup",
            Self::NonScriptSetup => "plain",
            Self::Universal => "universal",
            Self::ClientOnly => "client",
            Self::JsGlobalUniversal => "universal",
            Self::JsGlobalBrowser => "client",
            Self::JsGlobalNode => "server",
            Self::JsGlobalDeno => "server",
            Self::JsGlobalBun => "server",
            Self::VueGlobal => "vue",
            Self::ExternalModule => "extern",
            Self::Closure => "closure",
        }
    }

    /// Get reference prefix for parent scope references
    /// - `~` = universal (works on both client and server)
    /// - `!` = client only (requires client API)
    /// - `#` = server private (reserved for future Server Components)
    #[inline]
    pub const fn ref_prefix(&self) -> &'static str {
        match self {
            Self::ClientOnly | Self::JsGlobalBrowser => "!",
            Self::JsGlobalNode | Self::JsGlobalDeno | Self::JsGlobalBun => "#",
            _ => "~",
        }
    }
}
