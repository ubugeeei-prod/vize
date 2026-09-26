use super::{
    Result, cache, edits,
    schema::{Batch, Cost, Identity, SCHEMA, STAGE},
    walk,
};
use std::time::Instant;
use vize_atelier_core::{
    PropNode, TemplateChildNode, TextNode,
    codegen::{CodegenResult, generate},
    options::{CodegenMode, CodegenOptions, TransformOptions},
};
use vize_davinci::pass::NoObserver;
use vize_l0::Allocator;
use vize_l1_to_l2::{DomEmitMode, DomEmitOptions};

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct CompileOptions<'a> {
    pub source_map: bool,
    pub hoist_static: bool,
    pub cache: bool,
    pub cache_dir: Option<&'a std::path::Path>,
}
pub(crate) struct Output {
    pub result: CodegenResult,
    pub costs: Vec<Cost>,
}

pub(crate) fn compile(
    source: &str,
    filename: &str,
    plugins: &[Identity],
    options: CompileOptions<'_>,
    mut invoke: impl FnMut(usize, String) -> Result<String>,
) -> Result<Output> {
    if source.len() > 1_048_576 || filename.len() > 4096 || plugins.len() > 32 {
        return Err("transform input exceeds the source/plugin limit".into());
    }
    let allocator = Allocator::new();
    let (tree, errors) = vize_l1::parse(&allocator, source);
    let mut lowered = vize_l1_to_l2::lower(&allocator, &tree, &errors);
    if !lowered.diagnostics.is_empty() {
        return Err("transform requires a diagnostic-free L2 lowering".into());
    }
    // This original-source compatibility tree supplies the established maps.
    // Typed edits are mirrored before canonicalization; no edited string parse.
    let (mut compat, errors) = vize_atelier_core::parser::parse(&allocator, source);
    if !errors.is_empty() {
        return Err(format!(
            "transform compatibility parse rejected the source: {errors:?}"
        ));
    }
    let mut costs = Vec::new();
    let mut audited = Vec::new();
    for (index, plugin) in plugins.iter().enumerate() {
        let started = Instant::now();
        if plugin.name.is_empty()
            || plugin.name.len() > 128
            || plugin.version.is_empty()
            || plugin.version.len() > 128
            || plugin.fingerprint.is_empty()
            || plugin.fingerprint.len() > 256
        {
            return Err("transform identity requires bounded name, version and fingerprint".into());
        }
        if plugin.cache_inputs.as_ref().is_some_and(|inputs| {
            inputs.len() > 64
                || inputs
                    .iter()
                    .any(|(name, value)| name.len() > 128 || value.len() > 4096)
        }) {
            return Err("transform cacheInputs exceed the count or string limit".into());
        }
        if options.cache && plugin.cache_inputs.is_none() {
            return Err("transform caching requires explicit cacheInputs".into());
        }
        let nodes = walk::nodes(&mut lowered.root);
        let count = nodes.len() as u32;
        let batch = serde_json::to_string(&Batch {
            schema: SCHEMA,
            stage: STAGE,
            plugin: &plugin.name,
            file: filename,
            nodes,
        })
        .map_err(|e| e.to_string())?;
        let config = format!(
            "static-attribute-transform-v1;module;prefix=true;map={};hoist={};legacy={};glyph={}",
            options.source_map,
            options.hoist_static,
            cfg!(feature = "legacy"),
            cfg!(feature = "glyph")
        );
        let key = cache::key(source, filename, &batch, plugin, &config)?;
        let found = options
            .cache
            .then(|| cache::get(&key, options.cache_dir))
            .flatten()
            .filter(|reply| edits::validate(reply, &walk::nodes(&mut lowered.root)).is_ok());
        let cached = found.is_some();
        let mut js_ns = 0.0;
        let reply = match found {
            Some(reply) => reply,
            None => {
                let called = Instant::now();
                let first = invoke(index, batch.clone())?;
                js_ns += called.elapsed().as_nanos() as f64;
                let first = edits::decode(&first)?;
                let called = Instant::now();
                let second = invoke(index, batch)?;
                js_ns += called.elapsed().as_nanos() as f64;
                let second = edits::decode(&second)?;
                if first != second {
                    return Err(format!(
                        "{}: nondeterministic transform output",
                        plugin.name
                    ));
                }
                first
            }
        };
        // Cache hits still validate against the exact current artifact.
        edits::validate(&reply, &walk::nodes(&mut lowered.root))?;
        let applied = edits::apply(&mut lowered, &reply, &plugin.name);
        mirror(&allocator, &mut compat.children, &applied);
        if options.cache && !cached {
            audited.push((key.clone(), reply.clone()));
        }
        costs.push(Cost {
            name: plugin.name.clone(),
            content_key: key,
            nodes: count,
            edits: reply.edits.len() as u32,
            cached,
            elapsed_ns: started.elapsed().as_nanos() as f64,
            js_ns,
        });
    }
    // No analysis product exists before the hooks. Every pass observes the
    // edited native arena; stale hoist/slot/model products cannot survive.
    let profile = if options.hoist_static {
        vize_l1_to_l2::pass::TransformProfile::DEFAULT
    } else {
        vize_l1_to_l2::pass::TransformProfile::DEFAULT.without_static_analysis()
    };
    let facts =
        vize_l1_to_l2::pass::run_dom_transform_with_profile(&mut lowered, &mut NoObserver, profile);
    let emitted = vize_l1_to_l2::emit_dom_with_options(
        &lowered,
        &facts,
        &DomEmitOptions {
            mode: DomEmitMode::Module,
            prefix_identifiers: true,
            hoist_static: options.hoist_static,
            ..DomEmitOptions::DEFAULT
        },
    )
    .map_err(|e| format!("transformed L2 emission rejected: {e:?}"))?;
    let diagnostics = vize_atelier_core::transform(
        &allocator,
        &mut compat,
        TransformOptions {
            prefix_identifiers: true,
            hoist_static: options.hoist_static,
            ..Default::default()
        },
        None,
    );
    if !diagnostics.is_empty() {
        return Err("transformed compatibility consumer rejected the artifact".into());
    }
    let result = generate(
        &compat,
        CodegenOptions {
            mode: CodegenMode::Module,
            prefix_identifiers: true,
            source_map: options.source_map,
            filename: filename.into(),
            ..Default::default()
        },
    );
    if emitted.code != result.code || emitted.preamble != result.preamble {
        return Err("transformed L2 output differs from the compatibility consumer".into());
    }
    for (key, reply) in audited {
        cache::put(key, reply, options.cache_dir);
    }
    Ok(Output {
        result: CodegenResult {
            code: emitted.code,
            preamble: emitted.preamble,
            map: result.map,
        },
        costs,
    })
}

fn mirror<'a>(
    allocator: &'a Allocator,
    children: &mut [TemplateChildNode<'a>],
    applied: &[edits::Applied],
) {
    vize_l0::ensure_sufficient_stack(|| {
        for child in children {
            let TemplateChildNode::Element(element) = child else {
                continue;
            };
            let owner_start = element.loc.span.start;
            for edit in applied
                .iter()
                .filter(|edit| edit.owner_start == owner_start)
            {
                for prop in element.props.iter_mut() {
                    let PropNode::Attribute(attr) = prop else {
                        continue;
                    };
                    if attr.name == edit.name {
                        let loc = attr
                            .value
                            .as_ref()
                            .map_or_else(|| attr.loc.clone(), |value| value.loc.clone());
                        attr.value = edit
                            .value
                            .as_deref()
                            .map(|value| TextNode::new(allocator.alloc_str(value), loc));
                    }
                }
            }
            mirror(allocator, &mut element.children, applied);
        }
    });
}
