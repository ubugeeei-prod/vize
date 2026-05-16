//! NAPI bindings for Musea (Art file) features.
//!
//! Provides art parsing, CSF transformation, documentation generation,
//! catalog generation, palette/props controls, and variant autogeneration.
//!
//! FFI boundary code: uses std types for JavaScript interop.
#![allow(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros
)]

use napi::bindgen_prelude::{Error, Result, Status};
use napi_derive::napi;
use rayon::prelude::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use vize_carton::cstr;

// ============================================================================
// Art file types
// ============================================================================

/// Art parse options for NAPI
#[napi(object)]
#[derive(Default)]
pub struct ArtParseOptionsNapi {
    pub filename: Option<String>,
}

/// Art metadata for NAPI
#[napi(object)]
pub struct ArtMetadataNapi {
    pub title: String,
    pub description: Option<String>,
    pub component: Option<String>,
    pub category: Option<String>,
    pub tags: Vec<String>,
    pub status: String,
    pub order: Option<u32>,
}

/// Art variant for NAPI
#[napi(object)]
pub struct ArtVariantNapi {
    pub name: String,
    pub template: String,
    pub is_default: bool,
    pub skip_vrt: bool,
}

/// Art descriptor for NAPI
#[napi(object)]
pub struct ArtDescriptorNapi {
    pub filename: String,
    pub metadata: ArtMetadataNapi,
    pub variants: Vec<ArtVariantNapi>,
    pub has_script_setup: bool,
    pub has_script: bool,
    pub style_count: u32,
}

/// CSF output for NAPI
#[napi(object)]
pub struct CsfOutputNapi {
    pub code: String,
    pub filename: String,
}

// ============================================================================
// Doc types
// ============================================================================

/// Doc options for NAPI
#[napi(object)]
#[derive(Default)]
pub struct DocOptionsNapi {
    pub include_source: Option<bool>,
    pub include_templates: Option<bool>,
    pub include_metadata: Option<bool>,
    pub include_toc: Option<bool>,
    pub toc_threshold: Option<u32>,
    pub base_path: Option<String>,
    pub title: Option<String>,
}

/// Doc output for NAPI
#[napi(object)]
pub struct DocOutputNapi {
    pub markdown: String,
    pub filename: String,
    pub title: String,
    pub category: Option<String>,
    pub variant_count: u32,
}

/// Catalog entry for NAPI
#[napi(object)]
pub struct CatalogEntryNapi {
    pub title: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub tags: Vec<String>,
    pub status: String,
    pub variant_count: u32,
    pub doc_path: String,
    pub source_path: String,
}

/// Catalog output for NAPI
#[napi(object)]
pub struct CatalogOutputNapi {
    pub markdown: String,
    pub filename: String,
    pub component_count: u32,
    pub categories: Vec<String>,
    pub tags: Vec<String>,
}

// ============================================================================
// Palette types
// ============================================================================

/// Palette options for NAPI
#[napi(object)]
#[derive(Default)]
pub struct PaletteOptionsNapi {
    pub infer_options: Option<bool>,
    pub min_select_values: Option<u32>,
    pub max_select_values: Option<u32>,
    pub group_by_type: Option<bool>,
}

/// Select option for NAPI
#[napi(object)]
pub struct SelectOptionNapi {
    pub label: String,
    pub value: serde_json::Value,
}

/// Range config for NAPI
#[napi(object)]
pub struct RangeConfigNapi {
    pub min: f64,
    pub max: f64,
    pub step: Option<f64>,
}

/// Prop control for NAPI
#[napi(object)]
pub struct PropControlNapi {
    pub name: String,
    pub control: String,
    pub default_value: Option<serde_json::Value>,
    pub description: Option<String>,
    pub required: bool,
    pub options: Vec<SelectOptionNapi>,
    pub range: Option<RangeConfigNapi>,
    pub group: Option<String>,
}

/// Palette output for NAPI
#[napi(object)]
pub struct PaletteOutputNapi {
    pub title: String,
    pub controls: Vec<PropControlNapi>,
    pub groups: Vec<String>,
    pub json: String,
    pub typescript: String,
}

// ============================================================================
// Autogen types
// ============================================================================

#[napi(object)]
#[derive(Default)]
pub struct AutogenConfigNapi {
    pub max_variants: Option<u32>,
    pub include_default: Option<bool>,
    pub include_boolean_toggles: Option<bool>,
    pub include_enum_variants: Option<bool>,
    pub include_boundary_values: Option<bool>,
    pub include_empty_strings: Option<bool>,
}

#[napi(object)]
pub struct PropDefinitionNapi {
    pub name: String,
    pub prop_type: String,
    pub required: bool,
    pub default_value: Option<serde_json::Value>,
}

#[napi(object)]
pub struct GeneratedVariantNapi {
    pub name: String,
    pub is_default: bool,
    pub props: serde_json::Value,
    pub description: Option<String>,
}

#[napi(object)]
pub struct AutogenOutputNapi {
    pub variants: Vec<GeneratedVariantNapi>,
    pub art_file_content: String,
    pub component_name: String,
}

// ============================================================================
// Art functions
// ============================================================================

/// Parse Art file (*.art.vue)
#[napi(js_name = "parseArt")]
pub fn parse_art(
    source: String,
    options: Option<ArtParseOptionsNapi>,
) -> Result<ArtDescriptorNapi> {
    use vize_musea::{parse_art as musea_parse, ArtParseOptions, ArtStatus, Bump};

    let allocator = Bump::new();
    let opts = options.unwrap_or_default();
    let parse_opts = ArtParseOptions {
        filename: opts
            .filename
            .unwrap_or_else(|| "anonymous.art.vue".to_string())
            .into(),
    };

    let descriptor = musea_parse(&allocator, &source, parse_opts)
        .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;

    // Convert to owned types before allocator is dropped
    let metadata = ArtMetadataNapi {
        title: descriptor.metadata.title.to_string(),
        description: descriptor.metadata.description.map(|d| d.to_string()),
        component: descriptor.metadata.component.map(|c| c.to_string()),
        category: descriptor.metadata.category.map(|c| c.to_string()),
        tags: descriptor
            .metadata
            .tags
            .iter()
            .map(|t| t.to_string())
            .collect(),
        status: match descriptor.metadata.status {
            ArtStatus::Draft => "draft".to_string(),
            ArtStatus::Ready => "ready".to_string(),
            ArtStatus::Deprecated => "deprecated".to_string(),
        },
        order: descriptor.metadata.order,
    };

    let variants: Vec<ArtVariantNapi> = descriptor
        .variants
        .iter()
        .map(|v| ArtVariantNapi {
            name: v.name.to_string(),
            template: v.template.to_string(),
            is_default: v.is_default,
            skip_vrt: v.skip_vrt,
        })
        .collect();

    let result = ArtDescriptorNapi {
        filename: descriptor.filename.to_string(),
        metadata,
        variants,
        has_script_setup: descriptor.script_setup.is_some(),
        has_script: descriptor.script.is_some(),
        style_count: descriptor.styles.len() as u32,
    };

    // descriptor is dropped here, then allocator is dropped
    Ok(result)
}

/// Transform Art to Storybook CSF 3.0
#[napi(js_name = "artToCsf")]
pub fn art_to_csf(source: String, options: Option<ArtParseOptionsNapi>) -> Result<CsfOutputNapi> {
    use vize_musea::{parse_art as musea_parse, transform_to_csf, ArtParseOptions, Bump};

    let allocator = Bump::new();
    let opts = options.unwrap_or_default();
    let parse_opts = ArtParseOptions {
        filename: opts
            .filename
            .unwrap_or_else(|| "anonymous.art.vue".to_string())
            .into(),
    };

    let descriptor = musea_parse(&allocator, &source, parse_opts)
        .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;

    // transform_to_csf returns owned CsfOutput, so this is safe
    let csf = transform_to_csf(&descriptor);

    // Create result before descriptor and allocator are dropped
    let result = CsfOutputNapi {
        code: csf.code.into(),
        filename: csf.filename.into(),
    };

    Ok(result)
}

/// Generate component documentation from Art source
#[napi(js_name = "generateArtDoc")]
pub fn generate_art_doc(
    source: String,
    art_options: Option<ArtParseOptionsNapi>,
    doc_options: Option<DocOptionsNapi>,
) -> Result<DocOutputNapi> {
    use vize_musea::docs::{generate_component_doc, DocOptions};
    use vize_musea::{parse_art as musea_parse, ArtParseOptions, Bump};

    let allocator = Bump::new();
    let art_opts = art_options.unwrap_or_default();
    let parse_opts = ArtParseOptions {
        filename: art_opts
            .filename
            .unwrap_or_else(|| "anonymous.art.vue".to_string())
            .into(),
    };

    let descriptor = musea_parse(&allocator, &source, parse_opts)
        .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;

    let doc_opts = doc_options.unwrap_or_default();
    let opts = DocOptions {
        include_source: doc_opts.include_source.unwrap_or(false),
        include_templates: doc_opts.include_templates.unwrap_or(true),
        include_metadata: doc_opts.include_metadata.unwrap_or(true),
        include_toc: doc_opts.include_toc.unwrap_or(true),
        toc_threshold: doc_opts.toc_threshold.unwrap_or(5) as usize,
        base_path: doc_opts.base_path.unwrap_or_default().into(),
        title: doc_opts.title.map(Into::into),
        include_timestamp: false,
    };

    let output = generate_component_doc(&descriptor, &opts);

    Ok(DocOutputNapi {
        markdown: output.markdown.into(),
        filename: output.filename.into(),
        title: output.title.into(),
        category: output.category.map(Into::into),
        variant_count: output.variant_count as u32,
    })
}

/// Generate catalog from multiple Art sources (high-performance batch)
#[napi(js_name = "generateArtCatalog")]
pub fn generate_art_catalog(
    sources: Vec<String>,
    doc_options: Option<DocOptionsNapi>,
) -> Result<CatalogOutputNapi> {
    use vize_musea::docs::{generate_catalog, CatalogEntry, DocOptions};
    use vize_musea::{parse_art as musea_parse, ArtParseOptions, Bump};

    // Single allocator for all parses - efficient memory usage
    let allocator = Bump::new();

    // Parse all sources and collect entries
    let mut entries = Vec::with_capacity(sources.len());
    for (idx, source) in sources.iter().enumerate() {
        let parse_opts = ArtParseOptions {
            filename: cstr!("component_{idx}.art.vue"),
        };

        if let Ok(descriptor) = musea_parse(&allocator, source, parse_opts) {
            entries.push(CatalogEntry::from_descriptor(&descriptor, ""));
        }
    }

    let doc_opts = doc_options.unwrap_or_default();
    let opts = DocOptions {
        include_source: doc_opts.include_source.unwrap_or(false),
        include_templates: doc_opts.include_templates.unwrap_or(true),
        include_metadata: doc_opts.include_metadata.unwrap_or(true),
        include_toc: doc_opts.include_toc.unwrap_or(true),
        toc_threshold: doc_opts.toc_threshold.unwrap_or(5) as usize,
        base_path: doc_opts.base_path.unwrap_or_default().into(),
        title: doc_opts.title.map(Into::into),
        include_timestamp: false,
    };

    let output = generate_catalog(&entries, &opts);

    Ok(CatalogOutputNapi {
        markdown: output.markdown.into(),
        filename: output.filename.into(),
        component_count: output.component_count as u32,
        categories: output.categories.into_iter().map(Into::into).collect(),
        tags: output.tags.into_iter().map(Into::into).collect(),
    })
}

/// Batch generate docs with parallel processing
#[napi(js_name = "generateArtDocsBatch")]
pub fn generate_art_docs_batch(
    sources: Vec<String>,
    doc_options: Option<DocOptionsNapi>,
) -> Result<Vec<DocOutputNapi>> {
    use vize_musea::docs::{generate_component_doc, DocOptions};
    use vize_musea::{parse_art as musea_parse, ArtParseOptions, Bump};

    let doc_opts = doc_options.unwrap_or_default();
    let opts = DocOptions {
        include_source: doc_opts.include_source.unwrap_or(false),
        include_templates: doc_opts.include_templates.unwrap_or(true),
        include_metadata: doc_opts.include_metadata.unwrap_or(true),
        include_toc: doc_opts.include_toc.unwrap_or(true),
        toc_threshold: doc_opts.toc_threshold.unwrap_or(5) as usize,
        base_path: doc_opts.base_path.unwrap_or_default().into(),
        title: doc_opts.title.map(Into::into),
        include_timestamp: false,
    };

    // Process in parallel using rayon
    let results: Vec<DocOutputNapi> = sources
        .par_iter()
        .enumerate()
        .filter_map(|(idx, source)| {
            let allocator = Bump::new();
            let parse_opts = ArtParseOptions {
                filename: cstr!("component_{idx}.art.vue"),
            };

            musea_parse(&allocator, source, parse_opts)
                .ok()
                .map(|descriptor| {
                    let output = generate_component_doc(&descriptor, &opts);
                    DocOutputNapi {
                        markdown: output.markdown.into(),
                        filename: output.filename.into(),
                        title: output.title.into(),
                        category: output.category.map(Into::into),
                        variant_count: output.variant_count as u32,
                    }
                })
        })
        .collect();

    Ok(results)
}

/// Generate props palette from Art source
#[napi(js_name = "generateArtPalette")]
pub fn generate_art_palette(
    source: String,
    art_options: Option<ArtParseOptionsNapi>,
    palette_options: Option<PaletteOptionsNapi>,
) -> Result<PaletteOutputNapi> {
    use vize_musea::palette::{generate_palette, ControlKind, PaletteOptions};
    use vize_musea::{parse_art as musea_parse, ArtParseOptions, Bump};

    let allocator = Bump::new();
    let art_opts = art_options.unwrap_or_default();
    let parse_opts = ArtParseOptions {
        filename: art_opts
            .filename
            .unwrap_or_else(|| "anonymous.art.vue".to_string())
            .into(),
    };

    let descriptor = musea_parse(&allocator, &source, parse_opts)
        .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;

    let pal_opts = palette_options.unwrap_or_default();
    let opts = PaletteOptions {
        infer_options: pal_opts.infer_options.unwrap_or(true),
        min_select_values: pal_opts.min_select_values.unwrap_or(2) as usize,
        max_select_values: pal_opts.max_select_values.unwrap_or(10) as usize,
        group_by_type: pal_opts.group_by_type.unwrap_or(false),
    };

    let output = generate_palette(&descriptor, &opts);

    // Convert controls to NAPI types
    let controls: Vec<PropControlNapi> = output
        .palette
        .controls
        .iter()
        .map(|c| PropControlNapi {
            name: c.name.clone().into(),
            control: match c.control {
                ControlKind::Text => "text".to_string(),
                ControlKind::Number => "number".to_string(),
                ControlKind::Boolean => "boolean".to_string(),
                ControlKind::Range => "range".to_string(),
                ControlKind::Select => "select".to_string(),
                ControlKind::Radio => "radio".to_string(),
                ControlKind::Color => "color".to_string(),
                ControlKind::Date => "date".to_string(),
                ControlKind::Object => "object".to_string(),
                ControlKind::Array => "array".to_string(),
                ControlKind::File => "file".to_string(),
                ControlKind::Raw => "raw".to_string(),
            },
            default_value: c.default_value.clone(),
            description: c.description.clone().map(Into::into),
            required: c.required,
            options: c
                .options
                .iter()
                .map(|o| SelectOptionNapi {
                    label: o.label.clone().into(),
                    value: o.value.clone(),
                })
                .collect(),
            range: c.range.as_ref().map(|r| RangeConfigNapi {
                min: r.min,
                max: r.max,
                step: r.step,
            }),
            group: c.group.clone().map(Into::into),
        })
        .collect();

    Ok(PaletteOutputNapi {
        title: output.palette.title.into(),
        controls,
        groups: output.palette.groups.into_iter().map(Into::into).collect(),
        json: output.json.into(),
        typescript: output.typescript.into(),
    })
}

/// Generate .art.vue variants from component prop definitions
#[napi(js_name = "generateVariants")]
pub fn generate_variants(
    component_path: String,
    props: Vec<PropDefinitionNapi>,
    config: Option<AutogenConfigNapi>,
) -> Result<AutogenOutputNapi> {
    use vize_musea::autogen::{generate_art_file, AutogenConfig, PropDefinition};

    let cfg = config.unwrap_or_default();
    let autogen_config = AutogenConfig {
        max_variants: cfg.max_variants.unwrap_or(20) as usize,
        include_default: cfg.include_default.unwrap_or(true),
        include_boolean_toggles: cfg.include_boolean_toggles.unwrap_or(true),
        include_enum_variants: cfg.include_enum_variants.unwrap_or(true),
        include_boundary_values: cfg.include_boundary_values.unwrap_or(false),
        include_empty_strings: cfg.include_empty_strings.unwrap_or(false),
    };

    let prop_defs: Vec<PropDefinition> = props
        .into_iter()
        .map(|p| PropDefinition {
            name: p.name.into(),
            prop_type: p.prop_type.into(),
            required: p.required,
            default_value: p.default_value,
        })
        .collect();

    let output = generate_art_file(&component_path, &prop_defs, &autogen_config);

    let variants_napi: Vec<GeneratedVariantNapi> = output
        .variants
        .into_iter()
        .map(|v| GeneratedVariantNapi {
            name: v.name.into(),
            is_default: v.is_default,
            props: serde_json::Value::Object(v.props),
            description: v.description.map(Into::into),
        })
        .collect();

    Ok(AutogenOutputNapi {
        variants: variants_napi,
        art_file_content: output.art_file_content.into(),
        component_name: output.component_name.into(),
    })
}
