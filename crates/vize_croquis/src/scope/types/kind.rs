/// Kind of scope
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ScopeKind {
    /// SFC (Single File Component) level scope
    /// This is the root scope that contains script setup/non-script-setup scopes
    Module = 0,
    /// Function scope
    Function = 1,
    /// Block scope (if, for, etc.)
    Block = 2,
    /// v-for scope (template)
    VFor = 3,
    /// v-slot scope (template)
    VSlot = 4,
    /// Event handler scope (@click, etc.)
    EventHandler = 5,
    /// Callback/arrow function scope in expressions
    Callback = 6,
    /// Script setup scope (`<script setup>`)
    ScriptSetup = 7,
    /// Non-script setup scope (Options API, regular `<script>`)
    NonScriptSetup = 8,
    /// Universal scope (SSR - runs on both server and client)
    Universal = 9,
    /// Client-only scope (onMounted, onBeforeUnmount, etc.)
    ClientOnly = 10,
    /// Universal JavaScript global scope (console, Math, Object, Array, etc.)
    /// Works in all runtimes
    JsGlobalUniversal = 11,
    /// Browser-only JavaScript global scope (window, document, navigator, localStorage, etc.)
    /// WARNING: Not available in SSR server context
    JsGlobalBrowser = 12,
    /// Node.js-only JavaScript global scope (process, Buffer, __dirname, require, etc.)
    /// WARNING: Not available in browser context
    JsGlobalNode = 13,
    /// Deno-only JavaScript global scope (Deno namespace)
    JsGlobalDeno = 14,
    /// Bun-only JavaScript global scope (Bun namespace)
    JsGlobalBun = 15,
    /// Vue global scope ($refs, $emit, $slots, $attrs, etc.)
    VueGlobal = 16,
    /// External module scope (imported modules)
    ExternalModule = 17,
    /// Closure scope (function declaration, function expression, arrow function)
    /// Has access to arguments, this, and local variables
    Closure = 18,
    /// Patterned-template subject scope.
    VMatch = 19,
    /// A single pattern arm's lexical bindings.
    VWhen = 20,
}

impl ScopeKind {
    /// Get the display prefix for this scope kind
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
            Self::Module => "mod",
            Self::Function => "fn",
            Self::Block => "block",
            Self::VFor => "v-for",
            Self::VSlot => "v-slot",
            Self::EventHandler => "event",
            Self::Callback => "cb",
            Self::ScriptSetup => "setup",
            Self::NonScriptSetup => "plain",
            Self::Universal => "universal",
            Self::ClientOnly => "client",
            Self::JsGlobalUniversal => "univ",
            Self::JsGlobalBrowser => "client",
            Self::JsGlobalNode => "server",
            Self::JsGlobalDeno => "server",
            Self::JsGlobalBun => "server",
            Self::VueGlobal => "vue",
            Self::ExternalModule => "extern",
            Self::Closure => "closure",
            Self::VMatch => "v-match",
            Self::VWhen => "v-when",
        }
    }

    /// Format for VIR display (zero allocation)
    #[inline]
    pub const fn to_display(&self) -> &'static str {
        match self {
            Self::Module => "mod",
            Self::Function => "fn",
            Self::Block => "block",
            Self::VFor => "v-for",
            Self::VSlot => "v-slot",
            Self::EventHandler => "event",
            Self::Callback => "cb",
            Self::ScriptSetup => "setup",
            Self::NonScriptSetup => "plain",
            Self::Universal => "universal",
            Self::ClientOnly => "client",
            Self::JsGlobalUniversal => "univ",
            Self::JsGlobalBrowser => "client",
            Self::JsGlobalNode => "server",
            Self::JsGlobalDeno => "server",
            Self::JsGlobalBun => "server",
            Self::VueGlobal => "vue",
            Self::ExternalModule => "extern",
            Self::Closure => "closure",
            Self::VMatch => "v-match",
            Self::VWhen => "v-when",
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
