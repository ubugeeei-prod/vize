//! Input-dialect meaning that stays outside the neutral L2 op family.
//!
//! A document is not an SFC and does not enter the component lowering. Its
//! dialect-specific semantic views stay here instead of pretending to be a
//! component [`crate::artifact::Artifact`].

pub mod petite;

pub use petite::{
    PetiteDirective, PetiteDirectiveKind, PetiteDocument, PetiteDocumentRefusal, PetiteEffect,
    PetiteScope, PetiteScopeShape, PetiteValue,
};
