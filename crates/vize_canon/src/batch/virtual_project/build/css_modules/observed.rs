//! Build per-file checker options while other tools read the same authored style.
use super::{
    AuthoredClasses, CSS_MODULE_GLOBAL_MARKER, CssModuleShape, TemplateGlobal, VirtualTsOptions,
    contains_dynamic_css_module_exports, css_import_targets, css_module_type,
    virtual_ts_options_for_descriptor,
};
use std::collections::BTreeMap;
use vize_atelier_sfc::{SfcDescriptor, SfcStyleBlock};
use vize_l0::{SourceFrameError, SourceRoot};
use vize_l1::css::lang::{OriginalStyleStructure, StyleObservation};

impl VirtualTsOptions {
    /// The ordinary checker-only projection, including all source and project options.
    pub fn for_style_modules(
        &self,
        descriptor: &SfcDescriptor<'_>,
        strict: bool,
        resolve_imports: bool,
    ) -> Self {
        virtual_ts_options_for_descriptor(self, descriptor, strict, resolve_imports)
    }

    /// Read each CSS module's original structure and classes once. The callback
    /// also observes external sources, unknown dialects and incomplete class sets.
    /// An invalid source range stops the visit and discards the checker product.
    pub fn observe_style_modules(
        &self,
        descriptor: &SfcDescriptor<'_>,
        strict: bool,
        resolve_imports: bool,
        visit: impl FnMut(&str, &SfcStyleBlock<'_>, Option<&StyleObservation<'_>>),
    ) -> Result<VirtualTsOptions, SourceFrameError> {
        self.observe_styles::<false>(descriptor, strict, resolve_imports, |_, _| {}, visit)
    }

    /// Visit every original style in descriptor order while computing checker
    /// options. Inline module structure is shared with the callback. Ordinary
    /// styles require no class analysis for the checker.
    pub fn observe_file_styles(
        &self,
        descriptor: &SfcDescriptor<'_>,
        strict: bool,
        resolve_imports: bool,
        structure: impl FnMut(&SfcStyleBlock<'_>, Option<OriginalStyleStructure<'_, '_>>),
        module: impl FnMut(&str, &SfcStyleBlock<'_>, Option<&StyleObservation<'_>>),
    ) -> Result<VirtualTsOptions, SourceFrameError> {
        self.observe_styles::<true>(descriptor, strict, resolve_imports, structure, module)
    }

    fn observe_styles<const ALL: bool>(
        &self,
        descriptor: &SfcDescriptor<'_>,
        strict: bool,
        resolve_imports: bool,
        mut structure: impl FnMut(&SfcStyleBlock<'_>, Option<OriginalStyleStructure<'_, '_>>),
        mut module: impl FnMut(&str, &SfcStyleBlock<'_>, Option<&StyleObservation<'_>>),
    ) -> Result<VirtualTsOptions, SourceFrameError> {
        let root = SourceRoot::new(&descriptor.source)?;
        // Keep the common single module beside the walk. A B-tree node would
        // otherwise stay live while other consumers check the next style.
        let mut first: Option<(&str, Option<CssModuleShape>)> = None;
        let mut modules = BTreeMap::new();
        for style in &descriptor.styles {
            if !ALL && style.module.is_none() {
                continue;
            }
            let source = root
                .source()
                .get(style.loc.start..style.loc.end)
                .ok_or(SourceFrameError::BlockOutOfBounds)?;
            if !std::ptr::eq(source, style.content.as_ref()) && source != style.content.as_ref() {
                return Err(SourceFrameError::BlockNotRootSlice);
            }
            let block = root.block(source, style.loc.start as u32)?;
            let Some(name) = style.module.as_deref() else {
                structure(style, None);
                continue;
            };
            let original = match vize_l1::container::lang::style_lang(style.lang.as_deref()) {
                Ok(language) => Some(StyleObservation::read_with_structure(
                    language,
                    block,
                    |sheet| structure(style, Some(sheet)),
                )),
                Err(_) => {
                    structure(style, None);
                    None
                }
            };
            module(name, style, original.as_ref());
            let shape =
                original.and_then(|observed| module_shape(style, observed, resolve_imports));
            let Some((first_name, first_shape)) = first.as_mut() else {
                first = Some((name, shape));
                continue;
            };
            if *first_name == name {
                merge_shape(first_shape, shape);
            } else {
                match modules.entry(name) {
                    std::collections::btree_map::Entry::Vacant(entry) => {
                        entry.insert(shape);
                    }
                    std::collections::btree_map::Entry::Occupied(mut entry) => {
                        merge_shape(entry.get_mut(), shape)
                    }
                }
            }
        }
        if !modules.is_empty()
            && let Some((name, shape)) = first.take()
        {
            modules.insert(name, shape);
        }
        let mut globals = self.template_globals.clone();
        let mut unknown = if first.is_none() && modules.is_empty() {
            self.css_modules.clone()
        } else {
            Vec::new()
        };
        globals.retain(|global| {
            first
                .as_ref()
                .is_none_or(|(name, _)| *name != global.name.as_str())
                && !modules.contains_key(global.name.as_str())
        });
        for (name, shape) in first.into_iter().chain(modules) {
            if let Some(shape) = shape {
                globals.push(TemplateGlobal {
                    name: name.into(),
                    type_annotation: css_module_type(&shape, strict),
                    default_value: CSS_MODULE_GLOBAL_MARKER.into(),
                });
            } else {
                unknown.push(name.into());
            }
        }
        Ok(VirtualTsOptions {
            template_globals: globals,
            css_modules: unknown,
            auto_import_stubs: Vec::new(),
            auto_import_bindings: self.auto_import_binding_names(),
            external_template_bindings: self.external_template_bindings.clone(),
            reference_paths: self.reference_paths.clone(),
            strict_instance_globals: self.strict_instance_globals,
        })
    }
}

fn merge_shape(previous: &mut Option<CssModuleShape>, shape: Option<CssModuleShape>) {
    match (previous, shape) {
        (Some(previous), Some(shape)) => {
            previous.classes.extend(shape.classes);
            previous.imports.extend(shape.imports);
        }
        (previous, None) => *previous = None,
        (None, Some(_)) => {}
    }
}

fn module_shape(
    style: &SfcStyleBlock<'_>,
    observation: StyleObservation<'_>,
    resolve_imports: bool,
) -> Option<CssModuleShape> {
    if contains_dynamic_css_module_exports(&style.content) {
        return None;
    }
    let classes = observation
        .into_authored_classes()
        .filter(AuthoredClasses::is_complete)?
        .into_names();
    let mut imports = css_import_targets(&style.content);
    if !resolve_imports && (style.src.is_some() || !imports.is_empty()) {
        return None;
    }
    if let Some(src) = style.src.as_deref() {
        imports.insert(0, src.into());
    }
    Some(CssModuleShape { classes, imports })
}
