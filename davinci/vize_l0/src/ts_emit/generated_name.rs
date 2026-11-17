//! The one spelling of every name Vize writes into generated TypeScript.
//!
//! A generated type is `$` plus its real name in PascalCase (`$Slots`,
//! #11045); a generated value, property key, marker or parse probe is `$`
//! plus its real name in camelCase (`$templateInstance`). A family is a stem
//! that the producer completes: a number follows the stem directly
//! (`$propsNav0`), anything else follows an `_` (`$propCheck0_label`).
//!
//! Producers never spell a name themselves. A format string captures the
//! constant (`cstr!("{PROPS_NAV}{idx}")`), a `ts!` / `emit_ts!` hole splices
//! it, and a static text that must stay a literal is built with
//! [`generated_name!`]. Consumers recognise generated text through
//! [`GeneratedName::strip`] and [`is_generated`].
//!
//! A name never equals a Vue instance member (`$props`, `$attrs`, …, see
//! [`RESERVED`]). A name an SFC authors is taken by [`hygiene`]: while a
//! projection is bound, a colliding name is written with trailing `$`s until
//! no authored identifier has that spelling, and generated static text is
//! rewritten the same way by [`hygiene::push_text`].

use core::fmt;

use crate::ts_emit::{EmitHole, EmitLen, TsBuf};

pub mod hygiene;

/// A generated name or family stem. `Display` and the `ts!` holes write the
/// projection's spelling, which is the constant itself unless the SFC
/// authors it.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct GeneratedName(&'static str);

impl GeneratedName {
    /// The unbound spelling, for comparisons against text that was written
    /// without a projection.
    pub const fn as_str(self) -> &'static str {
        self.0
    }

    /// `text` after this name and any hygiene `$`s, when `text` starts with
    /// the name. A family member keeps its tail (`0_label`).
    pub fn strip(self, text: &str) -> Option<&str> {
        let rest = text.strip_prefix(self.0)?;
        Some(rest.trim_start_matches('$'))
    }

    /// Whether `text` is exactly this name, in any hygiene spelling.
    pub fn is(self, text: &str) -> bool {
        self.strip(text).is_some_and(str::is_empty)
    }

    /// Whether `text` is a member of this family: the stem, hygiene `$`s and
    /// a tail that starts with a digit or `_`.
    pub fn is_member(self, text: &str) -> bool {
        self.strip(text)
            .and_then(|tail| tail.bytes().next())
            .is_some_and(|byte| byte.is_ascii_digit() || byte == b'_')
    }

    /// Whether a member of this family starts anywhere in `text` at an
    /// identifier boundary.
    pub fn occurs_in(self, text: &str) -> bool {
        text.match_indices(self.0).any(|(start, _)| {
            let boundary = start
                .checked_sub(1)
                .and_then(|before| text.as_bytes().get(before))
                .is_none_or(|&byte| {
                    !(byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'$')
                });
            boundary
                && text.get(start..).is_some_and(|rest| {
                    self.is_member(rest)
                        || self.is(rest
                            .split(|c: char| !(c.is_ascii_alphanumeric() || c == '_' || c == '$'))
                            .next()
                            .unwrap_or_default())
                })
        })
    }

    /// Whether `text` ends with this name, in any hygiene spelling.
    pub fn ends(self, text: &str) -> bool {
        text.trim_end_matches('$')
            .strip_suffix(self.0)
            .is_some_and(|head| {
                head.bytes().next_back().is_none_or(|byte| {
                    !(byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'$')
                })
            })
    }

    /// The bound projection's spelling as a static string. A renamed
    /// spelling is interned once per process.
    pub fn spelled(self) -> &'static str {
        match self.fresh() {
            0 => self.0,
            count => hygiene::intern(self.0, count),
        }
    }

    fn fresh(self) -> usize {
        hygiene::fresh(self.0)
    }
}

impl fmt::Display for GeneratedName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.0)?;
        (0..self.fresh()).try_for_each(|_| formatter.write_str("$"))
    }
}

impl fmt::Debug for GeneratedName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, formatter)
    }
}

impl EmitLen for GeneratedName {
    #[inline]
    fn hole_len(&self) -> usize {
        self.0.len() + self.fresh()
    }
}

impl<B: TsBuf> EmitHole<B> for GeneratedName {
    #[inline]
    fn emit_hole(&self, buffer: &mut B) {
        buffer.push_ts_str(self.0);
        for _ in 0..self.fresh() {
            buffer.push_ts_str("$");
        }
    }
}

impl PartialEq<str> for GeneratedName {
    fn eq(&self, other: &str) -> bool {
        self.is(other)
    }
}

/// The literal spelling of a generated name: the only place a spelling is
/// written. Use it where a literal is required (`concat!`); elsewhere use the
/// [`GeneratedName`] constant of the same name.
#[macro_export]
#[rustfmt::skip]
macro_rules! generated_name {
    // Template context and instance receivers.
    // The authored-instance receiver template names are read from.
    (TEMPLATE_INSTANCE) => { "$inst" };
    // The public-instance receiver of a Vue 3 template without Options API.
    (PUBLIC_INSTANCE) => { "$public" };
    // The receiver of undeclared instance globals.
    (INSTANCE_GLOBALS) => { "$globals" };
    (STRICT_TEMPLATE_CONTEXT) => { "$strictCtx" };
    (KEYED_TEMPLATE_CONTEXT) => { "$keyedCtx" };
    // The Options API `this`.
    (OPTIONS_INSTANCE) => { "$this" };
    // The generated props object, when an authored binding takes `props`.
    (PROPS_OBJECT) => { "$propsObj" };

    // Fields of the setup and template records.
    (SETUP_PROPS) => { "$setupProps" };
    (OPTIONS_PROPS) => { "$optionsProps" };
    (TEMPLATE_REFS) => { "$refMap" };
    (ROOT_EL) => { "$rootEl" };
    // The template function's record.
    (TEMPLATE) => { "$template" };
    (TEMPLATE_SLOTS) => { "$templateSlots" };
    (TYPED_SLOTS) => { "$typedSlots" };
    (TYPED_EMITS) => { "$typedEmits" };
    (TYPED_EXPOSED) => { "$typedExposed" };
    (EXPOSED) => { "$exposed" };
    // The `defineEmits` call's value.
    (DEFINED_EMITS) => { "$emits" };
    (EMIT_OPTIONS) => { "$emitOptions" };
    (TEMPLATE_EMIT) => { "$templateEmit" };
    (EMIT_CALLS_NAV) => { "$emitCalls" };
    (FORWARDED) => { "$forwarded" };
    (MODEL_DEFAULTS) => { "$modelDefaults" };
    (WITH_DEFAULTS_PROPS) => { "$defaultedProps" };
    (PLAIN_SCRIPT_EXPORTS) => { "$scriptExports" };
    // The default export of an external `src` script.
    (SRC_DEFAULT) => { "$src" };
    // The default export of a Nuxt virtual module stub.
    (MODULE_DEFAULT) => { "$default" };
    (CSS_MODULE_GLOBAL) => { "$cssModule" };
    (TYPED_ROUTE_GLOBAL) => { "$typedRoute" };
    (RESOLVED_RUNTIME_DEFAULT) => { "$runtimeDefault" };

    // Listener and handler values.
    // A listener's arguments, `Parameters<listener>`.
    (LISTENER_ARGS) => { "$args" };
    // The parameter of a handler's identity wrapper.
    (CALLBACK) => { "$cb" };
    // The event value of a handler without Vue's implicit `$event`.
    (EVENT_ARG) => { "$evt" };
    (LISTENER) => { "$listener" };
    (MODEL_VALUE) => { "$value" };
    // The value alias of a JSX `v-for` without one.
    (FOR_VALUE) => { "$item" };

    // JSX render sinks.
    (JSX_EXPR) => { "$jsx" };
    (JSX_COMPONENT) => { "$jsxComponent" };
    (JSX_COMPONENT_SLOT) => { "$jsxSlot" };
    (JSX_COMPONENT_SPREAD) => { "$jsxSpread" };

    // Families: the producer appends the tail.
    (PROPS_NAV) => { "$propsNav" };
    (SLOTS_NAV) => { "$slotsNav" };
    (SLOT_NAV) => { "$slotNav" };
    (EVENTS_NAV) => { "$eventsNav" };
    (KEBAB_EVENTS_NAV) => { "$kebabNav" };
    (MODEL_EVENTS_NAV) => { "$modelNav" };
    (MODEL_EVENTS_COMPLETION) => { "$modelCompletion" };
    (EVENTS_RESOLVED) => { "$resolvedEvents" };
    (EVENT_KEYS) => { "$eventKeys" };
    (PROP_CHECK) => { "$prop" };
    (NATIVE_PROP_CHECK) => { "$nativeProp" };
    (COMPONENT_REF_CHECK) => { "$refCheck" };
    (DIRECTIVE_CHECK) => { "$directive" };
    (UNKNOWN_DIRECTIVE) => { "$unknownDirective" };
    (RESOLVED_PROPS) => { "$resolvedProps" };
    (SELECTED_PROPS) => { "$selectedProps" };
    (SLOT_HOST) => { "$slotHost" };
    (SLOT_PROPS_HOST) => { "$slotPropsHost" };
    (SLOT_PROPS) => { "$slotProps" };
    (SLOT_SCOPE) => { "$slotScope" };
    (SLOT_PAYLOAD) => { "$slotPayload" };
    (SLOT_NAME) => { "$slotName" };
    (SLOT_CHILDREN) => { "$slotChildren" };
    (REQUIRED_SLOTS) => { "$requiredSlots" };
    (V_FOR_SOURCE) => { "$forSource" };
    // A pattern-match binding of a structural template.
    (MATCH) => { "$match" };
    (HANDLER) => { "$handler" };
    (INLINE_HANDLER) => { "$inlineHandler" };
    (EMIT_FACTORY) => { "$emitFactory" };
    (EMIT_PROPS) => { "$emitProps" };
    (GLOBAL_COMPONENT) => { "$globalComponent" };
    (NATIVE_KEY) => { "$nativeKey" };
    (FRAGMENT_PROP) => { "$fragmentProp" };
    (TEMPLATE_REF) => { "$ref" };
    (GENERIC) => { "$generic" };
    (FORWARDED_PROPS) => { "$forwardedProps" };
    (FORWARDED_EMITS) => { "$forwardedEmits" };
    (DYNAMIC_GUARD) => { "$isGuard" };
    (DYNAMIC_GUARD_NOT) => { "$isNotGuard" };
    // The alias of a `<component :is>` value.
    (DYNAMIC_IS) => { "$is" };
    (SETUP_IMPORT) => { "$import" };
    (EXPORTED_TYPE) => { "$exportedType" };
    (AMBIENT) => { "$ambient" };
    (SHIPPED_DEFAULT) => { "$shippedDefault" };
    (OPTIONS_COMPUTED) => { "$computed" };
    (OPTIONS_METHOD) => { "$method" };
    (BOOLEAN_TARGET) => { "$booleanTarget" };
    (PATINA_PROMISE) => { "$promise" };
    (PATINA_PROP) => { "$propShape" };
    (PATINA_EMIT) => { "$emitCheck" };
    (PATINA_REACTIVITY) => { "$reactivity" };

    // Generated type aliases of one projection.
    (EVENTS_RESOLVER_TYPE) => { "$EventsResolver" };
    (LEGACY_PROPS_TYPE) => { "$LegacyProps" };
    (LEGACY_PROP_TYPE) => { "$LegacyProp" };
    (VNODE_HOOK_LISTENER_TYPE) => { "$VnodeHookListener" };
    (VNODE_HOOK_HANDLER_TYPE) => { "$VnodeHookHandler" };
    (TRANSITION_LISTENER_TYPE) => { "$TransitionListener" };
    (TRANSITION_HANDLER_TYPE) => { "$TransitionHandler" };
    (DYNAMIC_COMPONENT_LISTENER_TYPE) => { "$DynamicListener" };
    (EMIT_CLOSED_TYPE) => { "$EmitClosed" };
    (EMIT_CLOSED_FACTORY_TYPE) => { "$EmitClosedFactory" };
    (EMIT_RESOLVER_TYPE) => { "$EmitResolver" };
    (EMIT_INPUT_TYPE) => { "$EmitGate" };
    (EMIT_NATIVE_INPUT_TYPE) => { "$EmitNativeGate" };

    // Comment markers consumers find generated lines by.
    (TEMPLATE_USE_MARKER) => { "$use" };
    (TEMPLATE_HOVER_MARKER) => { "$hover" };
    (MODEL_EVENT_MARKER) => { "$modelEvent" };

    // Hosts of type-checked fragments.
    (PLAIN_HOST) => { "$plainHost" };
    (TEMPLATE_HANDLER) => { "$handlerHost" };

    // Parse-only wrappers; never type-checked.
    (SPREAD_PROBE) => { "$spread" };
    (HANDLER_PROBE) => { "$handlerBody" };
    (GENERIC_PROBE) => { "$typeParams" };
    (V_SCOPE_PROBE) => { "$scope" };
    (RUNTIME_PROBE) => { "$runtime" };
    (PRAGMA_PROBE) => { "$pragma" };
    (VALUE_PROBE) => { "$iterable" };
    (ASYNC_SETUP_PROBE) => { "$asyncSetup" };
    (DEFAULTS_PROBE) => { "$defaults" };
    (DEFAULT_PROBE) => { "$defaultValue" };
    (INCOMPLETE_MEMBER) => { "$incomplete" };
    (UNKNOWN_REFERENCE) => { "$unknown" };
}

macro_rules! generated_names {
    ($($name:ident),+ $(,)?) => {
        $(
            #[doc = concat!("`", generated_name!($name), "`")]
            pub const $name: GeneratedName = GeneratedName(generated_name!($name));
        )+

        /// Every generated name and family stem.
        pub const ALL: &[GeneratedName] = &[$($name),+];
    };
}

generated_names! {
    TEMPLATE_INSTANCE,
    PUBLIC_INSTANCE,
    INSTANCE_GLOBALS,
    STRICT_TEMPLATE_CONTEXT,
    KEYED_TEMPLATE_CONTEXT,
    OPTIONS_INSTANCE,
    PROPS_OBJECT,
    SETUP_PROPS,
    OPTIONS_PROPS,
    TEMPLATE_REFS,
    ROOT_EL,
    TEMPLATE,
    TEMPLATE_SLOTS,
    TYPED_SLOTS,
    TYPED_EMITS,
    TYPED_EXPOSED,
    EXPOSED,
    DEFINED_EMITS,
    EMIT_OPTIONS,
    TEMPLATE_EMIT,
    EMIT_CALLS_NAV,
    FORWARDED,
    MODEL_DEFAULTS,
    WITH_DEFAULTS_PROPS,
    PLAIN_SCRIPT_EXPORTS,
    SRC_DEFAULT,
    MODULE_DEFAULT,
    CSS_MODULE_GLOBAL,
    TYPED_ROUTE_GLOBAL,
    RESOLVED_RUNTIME_DEFAULT,
    LISTENER_ARGS,
    CALLBACK,
    EVENT_ARG,
    LISTENER,
    MODEL_VALUE,
    FOR_VALUE,
    JSX_EXPR,
    JSX_COMPONENT,
    JSX_COMPONENT_SLOT,
    JSX_COMPONENT_SPREAD,
    PROPS_NAV,
    SLOTS_NAV,
    SLOT_NAV,
    EVENTS_NAV,
    KEBAB_EVENTS_NAV,
    MODEL_EVENTS_NAV,
    MODEL_EVENTS_COMPLETION,
    EVENTS_RESOLVED,
    EVENT_KEYS,
    PROP_CHECK,
    NATIVE_PROP_CHECK,
    COMPONENT_REF_CHECK,
    DIRECTIVE_CHECK,
    UNKNOWN_DIRECTIVE,
    RESOLVED_PROPS,
    SELECTED_PROPS,
    SLOT_HOST,
    SLOT_PROPS_HOST,
    SLOT_PROPS,
    SLOT_SCOPE,
    SLOT_PAYLOAD,
    SLOT_NAME,
    SLOT_CHILDREN,
    REQUIRED_SLOTS,
    V_FOR_SOURCE,
    MATCH,
    HANDLER,
    INLINE_HANDLER,
    EMIT_FACTORY,
    EMIT_PROPS,
    GLOBAL_COMPONENT,
    NATIVE_KEY,
    FRAGMENT_PROP,
    TEMPLATE_REF,
    GENERIC,
    FORWARDED_PROPS,
    FORWARDED_EMITS,
    DYNAMIC_GUARD,
    DYNAMIC_GUARD_NOT,
    DYNAMIC_IS,
    SETUP_IMPORT,
    EXPORTED_TYPE,
    AMBIENT,
    SHIPPED_DEFAULT,
    OPTIONS_COMPUTED,
    OPTIONS_METHOD,
    BOOLEAN_TARGET,
    PATINA_PROMISE,
    PATINA_PROP,
    PATINA_EMIT,
    PATINA_REACTIVITY,
    EVENTS_RESOLVER_TYPE,
    LEGACY_PROPS_TYPE,
    LEGACY_PROP_TYPE,
    VNODE_HOOK_LISTENER_TYPE,
    VNODE_HOOK_HANDLER_TYPE,
    TRANSITION_LISTENER_TYPE,
    TRANSITION_HANDLER_TYPE,
    DYNAMIC_COMPONENT_LISTENER_TYPE,
    EMIT_CLOSED_TYPE,
    EMIT_CLOSED_FACTORY_TYPE,
    EMIT_RESOLVER_TYPE,
    EMIT_INPUT_TYPE,
    EMIT_NATIVE_INPUT_TYPE,
    TEMPLATE_USE_MARKER,
    TEMPLATE_HOVER_MARKER,
    MODEL_EVENT_MARKER,
    PLAIN_HOST,
    TEMPLATE_HANDLER,
    SPREAD_PROBE,
    HANDLER_PROBE,
    GENERIC_PROBE,
    V_SCOPE_PROBE,
    RUNTIME_PROBE,
    PRAGMA_PROBE,
    VALUE_PROBE,
    ASYNC_SETUP_PROBE,
    DEFAULTS_PROBE,
    DEFAULT_PROBE,
    INCOMPLETE_MEMBER,
    UNKNOWN_REFERENCE,
}

/// Vue instance members, template locals and the plugin globals the
/// generator models. No generated name or stem equals one.
pub const RESERVED: &[&str] = &[
    "$attrs",
    "$children",
    "$createElement",
    "$d",
    "$data",
    "$delete",
    "$destroy",
    "$el",
    "$emit",
    "$event",
    "$fetch",
    "$forceUpdate",
    "$host",
    "$i18n",
    "$id",
    "$isServer",
    "$listeners",
    "$mount",
    "$n",
    "$nextTick",
    "$nuxt",
    "$off",
    "$on",
    "$once",
    "$options",
    "$parent",
    "$props",
    "$refs",
    "$root",
    "$route",
    "$router",
    "$scopedSlots",
    "$set",
    "$slots",
    "$ssrContext",
    "$store",
    "$t",
    "$tc",
    "$te",
    "$vnode",
    "$watch",
];

/// Whether `ident` is a generated name or a member of a generated family,
/// in any hygiene spelling.
pub fn is_generated(ident: &str) -> bool {
    ident.starts_with('$')
        && ALL
            .iter()
            .any(|name| name.is(ident) || name.is_member(ident))
}

#[cfg(test)]
mod tests;
