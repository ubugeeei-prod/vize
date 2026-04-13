use super::{errors::compact_error, CorsaTypeAwareSession, TypeProbe};
use corsa::{api::TypeProbeOptions, runtime::block_on};
use vize_carton::{profile, String, ToCompactString};

impl CorsaTypeAwareSession {
    pub(in crate::linter) fn probe_type_at_offset(
        &self,
        generated_source: &str,
        generated_offset: u32,
        load_property_types: bool,
        load_signatures: bool,
    ) -> Result<Option<TypeProbe>, String> {
        let utf16_offset = profile!(
            "patina.corsa_session.byte_to_utf16",
            byte_offset_to_utf16_offset(generated_source, generated_offset)
        );
        profile!(
            "patina.corsa_session.probe_type",
            block_on(self.session.probe_type_at_position(
                self.virtual_file_wire.as_str(),
                utf16_offset,
                TypeProbeOptions {
                    load_property_types,
                    load_signatures,
                },
            ))
        )
        .map_err(|error| {
            compact_error(
                "Failed to query checker type probe",
                error.to_compact_string().as_str(),
            )
        })
    }
}

pub(super) fn byte_offset_to_utf16_offset(source: &str, byte_offset: u32) -> u32 {
    let mut clamped = usize::min(byte_offset as usize, source.len());
    while clamped > 0 && !source.is_char_boundary(clamped) {
        clamped -= 1;
    }
    source[..clamped].encode_utf16().count() as u32
}
