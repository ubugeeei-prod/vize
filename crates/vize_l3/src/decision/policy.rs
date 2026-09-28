//! Target policy identities for shared L3 analysis.
//!
//! The policy-specific criteria belong here, never in the L4 encoders.
//! Only their identities are established in this skeleton; criterion
//! evaluation remains part of issue #6839's unfinished producer.

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
