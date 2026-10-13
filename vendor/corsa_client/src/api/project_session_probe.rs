use corsa_core::fast::{CompactString, ToCompactString};

use super::{DocumentIdentifier, ProjectSession, TypeProbe, TypeProbeOptions, TypeResponse};
use crate::Result;

impl ProjectSession {
    /// Resolves a type probe for a UTF-16 position inside a document.
    ///
    /// This starts from the visible symbol when one exists, falls back to the
    /// direct type-at-position endpoint when needed, and optionally enriches
    /// the result with property and signature detail.
    pub async fn probe_type_at_position(
        &self,
        file: impl Into<DocumentIdentifier>,
        position: u32,
        options: TypeProbeOptions,
    ) -> Result<Option<TypeProbe>> {
        let file = file.into();
        let type_response =
            if let Some(symbol) = self.get_symbol_at_position(file.clone(), position).await? {
                match self.get_type_of_symbol(symbol.id).await? {
                    Some(type_response) => Some(type_response),
                    None => self.get_type_at_position(file.clone(), position).await?,
                }
            } else {
                self.get_type_at_position(file.clone(), position).await?
            };
        let Some(type_response) = type_response else {
            return Ok(None);
        };

        let mut probe = TypeProbe {
            type_texts: self.render_type_texts(&type_response).await?,
            property_names: Vec::new(),
            property_types: Vec::new(),
            call_signatures: Vec::new(),
            return_types: Vec::new(),
        };

        let properties = self
            .get_properties_of_type(type_response.id.clone())
            .await?;
        probe.property_names.reserve(properties.len());
        for property in &properties {
            probe
                .property_names
                .push(property.name.as_str().to_compact_string());
        }

        if options.load_property_types && !properties.is_empty() {
            let property_types = self
                .get_types_of_symbols(
                    properties
                        .iter()
                        .map(|property| property.id.clone())
                        .collect(),
                )
                .await?;

            probe.property_types.reserve(property_types.len());
            for property_type in property_types {
                if let Some(property_type) = property_type {
                    probe
                        .property_types
                        .push(self.render_type_texts(&property_type).await?);
                } else {
                    probe.property_types.push(Vec::new());
                }
            }
        }

        if options.load_signatures {
            let signatures = self.get_signatures_of_type(type_response.id, 0).await?;
            probe.call_signatures.reserve(signatures.len());
            probe.return_types.reserve(signatures.len());

            for signature in signatures {
                if signature.parameters.is_empty() {
                    probe.call_signatures.push(Vec::new());
                } else {
                    let parameter_types = self
                        .get_types_of_symbols(signature.parameters.clone())
                        .await?;
                    let mut rendered_parameters = Vec::with_capacity(parameter_types.len());

                    for parameter_type in parameter_types {
                        if let Some(parameter_type) = parameter_type {
                            rendered_parameters
                                .push(self.render_type_texts(&parameter_type).await?);
                        } else {
                            rendered_parameters.push(Vec::new());
                        }
                    }

                    probe.call_signatures.push(rendered_parameters);
                }

                if let Some(return_type) = self.get_return_type_of_signature(signature.id).await? {
                    probe
                        .return_types
                        .push(self.render_type_texts(&return_type).await?);
                } else {
                    probe.return_types.push(Vec::new());
                }
            }
        }

        Ok(Some(probe))
    }

    /// Renders all stable type texts currently known for a type.
    ///
    /// The `texts` already returned by Corsa are preferred, and the method
    /// falls back to `typeToString` only when those are absent.
    pub async fn render_type_texts(
        &self,
        type_response: &TypeResponse,
    ) -> Result<Vec<CompactString>> {
        let mut texts = Vec::with_capacity(type_response.texts.len() + 1);
        for text in &type_response.texts {
            push_unique_text(&mut texts, text.as_str());
        }

        if texts.is_empty() {
            let rendered = self
                .type_to_string(type_response.id.clone(), None, None)
                .await?;
            push_unique_text(&mut texts, rendered.as_str());
        }

        Ok(texts)
    }
}

fn push_unique_text(texts: &mut Vec<CompactString>, text: &str) {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return;
    }
    if texts.iter().any(|existing| existing.as_str() == trimmed) {
        return;
    }
    texts.push(trimmed.to_compact_string());
}
