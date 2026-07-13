//! CrossFileAnalyzer WASM bindings.
//!
//! FFI boundary code: uses std types for JavaScript interop.
#![allow(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros
)]

use super::{to_js_value, utf8_byte_to_utf16_offset};
use wasm_bindgen::prelude::*;

#[path = "cross_file/diagnostic_kind.rs"]
mod diagnostic_kind;

use diagnostic_kind::diagnostic_kind_to_string;

/// Analyze multiple Vue SFC files for cross-file issues
#[wasm_bindgen(js_name = "analyzeCrossFile")]
pub fn analyze_cross_file_wasm(files: JsValue, options: JsValue) -> Result<JsValue, JsValue> {
    use vize_atlas::Compilation;
    use vize_croquis_cf::{
        CrossFileAnalysisInput, CrossFileAnalysisProduct, CrossFileAnalysisRequest,
    };

    let cross_file_opts = parse_cross_file_options(&options);
    let files_array = js_sys::Array::from(&files);
    let mut file_data: Vec<(String, String)> = Vec::new();

    for i in 0..files_array.length() {
        let file_obj = files_array.get(i);
        let path = js_sys::Reflect::get(&file_obj, &JsValue::from_str("path"))
            .ok()
            .and_then(|v| v.as_string())
            .unwrap_or_else(|| format!("file_{i}.vue"));
        let source = js_sys::Reflect::get(&file_obj, &JsValue::from_str("source"))
            .ok()
            .and_then(|v| v.as_string())
            .unwrap_or_default();

        file_data.push((path, source));
    }

    let mut compilation = Compilation::new();
    vize_atelier_sfc::register_atlas_providers(&mut compilation)
        .map_err(|error| JsValue::from_str(&error.to_string()))?;
    vize_atelier_jsx::register_atlas_providers(&mut compilation)
        .map_err(|error| JsValue::from_str(&error.to_string()))?;
    vize_croquis_cf::register_atlas_provider(&mut compilation)
        .map_err(|error| JsValue::from_str(&error.to_string()))?;
    compilation
        .set_input::<CrossFileAnalysisInput>(CrossFileAnalysisRequest::new(cross_file_opts))
        .map_err(|error| JsValue::from_str(&error.to_string()))?;
    let mut source_to_index = std::collections::HashMap::new();
    let mut anchor = None;
    for (index, (path, source)) in file_data.iter().enumerate() {
        let source_id = compilation
            .add_source(path.as_str(), source.as_str())
            .map_err(|error| JsValue::from_str(&error.to_string()))?;
        anchor.get_or_insert(source_id);
        source_to_index.insert(source_id, index);
    }
    let anchor = match anchor {
        Some(anchor) => anchor,
        None => compilation
            .add_source("<cross-file-anchor>", "")
            .map_err(|error| JsValue::from_str(&error.to_string()))?,
    };
    let artifact = compilation
        .query::<CrossFileAnalysisProduct>(anchor)
        .map_err(|error| JsValue::from_str(&error.to_string()))?;
    let artifact = artifact.value();
    let result = artifact.result();
    let file_paths: Vec<_> = file_data.iter().map(|(path, _)| path.clone()).collect();
    let file_contents: Vec<_> = file_data.iter().map(|(_, source)| source.clone()).collect();

    let diagnostics: Vec<serde_json::Value> = result
        .diagnostics
        .iter()
        .map(|d| {
            let (primary_source, utf8_start, utf8_end) = artifact
                .diagnostic_range(d)
                .expect("analyzed diagnostic must retain an Atlas source layout");
            let primary_index = source_to_index.get(&primary_source).copied();
            let primary_file = primary_index
                .and_then(|index| file_paths.get(index))
                .cloned()
                .unwrap_or_default();
            let primary_content = primary_index
                .and_then(|index| file_contents.get(index))
                .map_or("", String::as_str);
            let adjusted_primary_offset = utf8_byte_to_utf16_offset(primary_content, utf8_start);
            let adjusted_primary_end_offset = utf8_byte_to_utf16_offset(primary_content, utf8_end);

            let related_locations: Vec<serde_json::Value> = d
                .related_files
                .iter()
                .filter_map(
                    |(file_id, offset, message): &(
                        vize_croquis_cf::FileId,
                        u32,
                        vize_carton::CompactString,
                    )| {
                        let (source, utf8_offset) =
                            artifact.diagnostic_related_offset(d, *file_id, *offset)?;
                        let index = source_to_index.get(&source).copied()?;
                        let file_path = file_paths.get(index)?.clone();
                        let related_content = file_contents.get(index).map_or("", String::as_str);
                        let adjusted_offset =
                            utf8_byte_to_utf16_offset(related_content, utf8_offset);

                        Some(serde_json::json!({
                            "file": file_path,
                            "offset": adjusted_offset,
                            "message": message.as_str(),
                        }))
                    },
                )
                .collect();

            let kind_str = diagnostic_kind_to_string(&d.kind);
            // Use the code() method from diagnostics.rs for unified code naming
            let code = d.code();

            serde_json::json!({
                "type": kind_str,
                "code": code,
                "severity": d.severity.display_name(),
                "message": d.message.as_str(),
                "file": primary_file,
                "offset": adjusted_primary_offset,
                "endOffset": adjusted_primary_end_offset,
                "relatedLocations": related_locations,
                "suggestion": d.suggestion.as_ref().map(|s| s.as_str()),
            })
        })
        .collect();

    let circular_deps: Vec<Vec<String>> = result
        .circular_deps
        .iter()
        .map(|cycle| {
            cycle
                .iter()
                .filter_map(|id| artifact.layout(*id).map(|layout| layout.path().to_owned()))
                .collect()
        })
        .collect();

    let output = serde_json::json!({
        "diagnostics": diagnostics,
        "circularDependencies": circular_deps,
        "complexityReport": super::cross_file_complexity::complexity_report_json(&result.complexity_report),
        "complexityHotspots": super::cross_file_complexity::complexity_hotspots_json(&result.complexity_hotspots),
        "stats": {
            "filesAnalyzed": result.stats.files_analyzed,
            "vueComponents": result.stats.vue_components,
            "dependencyEdges": result.stats.dependency_edges,
            "errorCount": result.stats.error_count,
            "warningCount": result.stats.warning_count,
            "infoCount": result.stats.info_count,
            "analysisTimeMs": result.stats.analysis_time_ms,
        },
        "filePaths": file_paths,
    });

    to_js_value(&output)
}

/// Parse CrossFileOptions from JsValue
fn parse_cross_file_options(options: &JsValue) -> vize_croquis_cf::CrossFileOptions {
    use vize_croquis_cf::CrossFileOptions;

    let get_bool = |key: &str| -> bool {
        js_sys::Reflect::get(options, &JsValue::from_str(key))
            .ok()
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    };

    let all_enabled = get_bool("all");
    if all_enabled {
        return CrossFileOptions::all();
    }

    CrossFileOptions {
        fallthrough_attrs: get_bool("fallthroughAttrs"),
        component_emits: get_bool("componentEmits"),
        event_bubbling: get_bool("eventBubbling"),
        provide_inject: get_bool("provideInject"),
        unique_ids: get_bool("uniqueIds"),
        server_client_boundary: get_bool("serverClientBoundary"),
        error_suspense_boundary: get_bool("errorSuspenseBoundary"),
        reactivity_tracking: get_bool("reactivityTracking"),
        race_conditions: get_bool("raceConditions"),
        setup_context: get_bool("setupContext"),
        circular_dependencies: get_bool("circularDependencies"),
        max_import_depth: js_sys::Reflect::get(options, &JsValue::from_str("maxImportDepth"))
            .ok()
            .and_then(|v| v.as_f64())
            .map(|v| v as usize),
        component_resolution: get_bool("componentResolution"),
        props_validation: get_bool("propsValidation"),
    }
}
