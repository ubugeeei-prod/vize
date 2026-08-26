//! Validated deserialization for the reporter descriptor wire shape.

use serde::{Deserialize, Deserializer, de};
use vize_s0::String;

use super::{ReporterAudience, ReporterCapability, ReporterDescriptor, ReporterTransport};

impl<'de> Deserialize<'de> for ReporterDescriptor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase", deny_unknown_fields)]
        struct Wire {
            contract_version: u32,
            id: String,
            display_name: String,
            format_version: u32,
            media_type: String,
            file_extension: Option<String>,
            transport: ReporterTransport,
            audiences: Vec<ReporterAudience>,
            capabilities: Vec<ReporterCapability>,
        }

        let wire = Wire::deserialize(deserializer)?;
        let descriptor = Self {
            contract_version: wire.contract_version,
            id: wire.id,
            display_name: wire.display_name,
            format_version: wire.format_version,
            media_type: wire.media_type,
            file_extension: wire.file_extension,
            transport: wire.transport,
            audiences: wire.audiences,
            capabilities: wire.capabilities,
        };
        descriptor.validate().map_err(de::Error::custom)?;
        Ok(descriptor)
    }
}
