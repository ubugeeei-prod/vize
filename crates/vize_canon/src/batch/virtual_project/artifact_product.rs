use vize_atlas::Product;
use vize_carton::{String as CompactString, cstr};

use crate::batch::error::{CorsaError, CorsaResult};

use super::build::RegisteredFile;

pub(super) struct CanonTypedDocumentProduct;

impl Product for CanonTypedDocumentProduct {
    type Value = CanonTypedDocumentArtifact;

    const NAME: &'static str = "canon.typed-document";
}

#[derive(Clone)]
pub(super) enum CanonTypedDocumentArtifact {
    Registered(RegisteredFile),
    SfcParse(CompactString),
    Failed(CompactString),
}

impl CanonTypedDocumentArtifact {
    pub(super) fn from_result(result: CorsaResult<RegisteredFile>) -> Self {
        match result {
            Ok(registered) => Self::Registered(registered),
            Err(error) => Self::Failed(cstr!("{error}")),
        }
    }

    pub(super) fn to_corsa_result(&self) -> CorsaResult<RegisteredFile> {
        match self {
            Self::Registered(registered) => Ok(registered.clone()),
            Self::SfcParse(message) => Err(CorsaError::SfcParse(message.clone())),
            Self::Failed(message) => Err(CorsaError::ArtifactGraph(message.clone())),
        }
    }
}
