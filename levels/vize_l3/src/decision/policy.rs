//! Target policy identities for shared L3 analysis.
//!
//! The policy-specific criteria belong here, never in the L4 encoders.
//! This bounded policy filters dynamic attached bindings only. Hoist/cache
//! eligibility and target-specific emission remain unfinished.

/// The canonical op that owns an attached binding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindingOwner {
    /// A native element.
    Element,
    /// A component reference.
    Component,
    /// A slot outlet's props surface.
    Slot,
}

/// The binding distinction needed by the admitted output policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindingRole {
    /// An event handler.
    Event,
    /// Compile-time cloak marker.
    Cloak,
    /// Every other attached binding, conservatively dynamic.
    Other,
}

/// Target whose eligibility criteria the L3 decision producer must apply.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetPolicy {
    /// DOM placement and dynamic-binding eligibility.
    Dom,
    /// Server rendering placement and ordering eligibility.
    Ssr,
    /// Reactive program placement and effect eligibility.
    Vapor,
}

impl TargetPolicy {
    /// Whether an attached binding affects this target's output.
    ///
    /// `v-cloak` is compile-time only. SSR omits native-element events,
    /// while component events and slot props remain dynamic. DOM and Vapor
    /// otherwise retain every binding; this does not claim effect lowering.
    #[must_use]
    pub const fn binding_is_dynamic(self, owner: BindingOwner, role: BindingRole) -> bool {
        !matches!(role, BindingRole::Cloak)
            && !matches!(
                (self, owner, role),
                (Self::Ssr, BindingOwner::Element, BindingRole::Event)
            )
    }
}

/// DOM decision policy namespace.
pub mod dom {
    use super::TargetPolicy;

    /// Select DOM-specific eligibility criteria in the native producer.
    pub const POLICY: TargetPolicy = TargetPolicy::Dom;
}

/// Server rendering decision policy namespace.
pub mod ssr {
    use super::TargetPolicy;

    /// Select server rendering criteria in the native producer.
    pub const POLICY: TargetPolicy = TargetPolicy::Ssr;
}

/// Reactive program decision policy namespace.
pub mod vapor {
    use super::TargetPolicy;

    /// Select reactive program criteria in the native producer.
    pub const POLICY: TargetPolicy = TargetPolicy::Vapor;
}

#[cfg(test)]
mod tests {
    use super::{BindingOwner, BindingRole, TargetPolicy};

    #[test]
    fn cloak_is_compile_time_for_every_owner_and_target() {
        for policy in [TargetPolicy::Dom, TargetPolicy::Ssr, TargetPolicy::Vapor] {
            for owner in [
                BindingOwner::Element,
                BindingOwner::Component,
                BindingOwner::Slot,
            ] {
                assert!(!policy.binding_is_dynamic(owner, BindingRole::Cloak));
                assert!(policy.binding_is_dynamic(owner, BindingRole::Other));
            }
        }
    }

    #[test]
    fn only_ssr_native_element_events_are_omitted() {
        for policy in [TargetPolicy::Dom, TargetPolicy::Ssr, TargetPolicy::Vapor] {
            for owner in [
                BindingOwner::Element,
                BindingOwner::Component,
                BindingOwner::Slot,
            ] {
                assert_eq!(
                    policy.binding_is_dynamic(owner, BindingRole::Event),
                    !(policy == TargetPolicy::Ssr && owner == BindingOwner::Element),
                );
            }
        }
    }
}
