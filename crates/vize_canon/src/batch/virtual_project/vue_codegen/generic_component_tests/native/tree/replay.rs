//! The before control changes no authored or mapped bytes.

use super::super::packets;
use crate::batch::virtual_project::VirtualFile;
use vize_carton::{String, cstr};

pub(super) fn without_event_spreads(
    file: &VirtualFile,
    expected: bool,
    navigation: bool,
    name: &str,
) -> String {
    let mut bytes = file.content.as_bytes().to_vec();
    let mut offset = 0;
    let mut inference = None;
    let mut erased = Vec::new();
    let mut kinds = [false; 2];
    for line in file.content.split_inclusive('\n') {
        for (index, marker) in ["const __vize_emit_props_", "const __vize_events_resolved_"]
            .iter()
            .enumerate()
        {
            if line.contains(marker) && line.trim_end().ends_with('{') {
                assert!(inference.is_none(), "ambiguous inference argument: {line}");
                inference = Some(index);
            }
        }
        if let Some(index) = inference {
            let trimmed = line.trim();
            if trimmed.starts_with("...") && trimmed.ends_with(".bind,") {
                let start = offset + line.find("...").unwrap();
                let end = offset + line.trim_end().len();
                assert!(
                    (start..end).all(|position| file
                        .source_map
                        .get_original_position(position as u32)
                        .is_none()),
                    "an inference replay must not change mapped authored bytes"
                );
                bytes[start..end].fill(b' ');
                erased.push((start, end, String::from(trimmed)));
                kinds[index] = true;
            }
            if trimmed == "});" {
                inference = None;
            }
        }
        offset += line.len();
    }
    let before = String::from(std::string::String::from_utf8(bytes).unwrap());
    packets::retain(
        &cstr!("tree-{name}-replay"),
        &serde_json::json!({
            "wholeProduction": file.content,"wholePriorArgumentOmission": before,
            "wholeUnchangedMaps":cstr!("{:#?}",file.source_map),"unmappedEqualWidthErasures":erased,
        }),
    );
    assert!(inference.is_none(), "unterminated actual inference object");
    assert_eq!(
        kinds,
        [expected, expected && navigation],
        "both existing event inference emitters must consume the same authored spread"
    );
    assert_eq!(before.len(), file.content.len());
    if !expected {
        assert_eq!(before, file.content);
    }
    before
}
