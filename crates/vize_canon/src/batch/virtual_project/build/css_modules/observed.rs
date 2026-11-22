//! Build per-file checker options while other tools read the same authored style.
use super::{
    AuthoredClasses, CSS_MODULE_GLOBAL_MARKER, CssModuleShape, TemplateGlobal, VirtualTsOptions,
    contains_dynamic_css_module_exports, css_import_targets, css_module_type,
    virtual_ts_options_for_descriptor,
};
use std::collections::{BTreeMap, btree_map::Entry};
use vize_atelier_sfc::{SfcDescriptor, SfcStyleBlock};
use vize_l0::{SourceFrameError, SourceRoot};
use vize_l1::css::lang::StyleObservation;

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
        mut visit: impl FnMut(&str, &SfcStyleBlock<'_>, Option<&StyleObservation<'_>>),
    ) -> Result<VirtualTsOptions, SourceFrameError> {
        self.observe_styles::<false>(descriptor, strict, resolve_imports, |style, observed| {
            if let Some(name) = style.module.as_deref() {
                visit(name, style, observed);
            }
        })
    }

    /// Visit every original style in descriptor order while computing checker
    /// options. Inline module structure is shared with the callback. Ordinary
    /// styles require no class analysis for the checker.
    pub fn observe_file_styles(
        &self,
        descriptor: &SfcDescriptor<'_>,
        strict: bool,
        resolve_imports: bool,
        visit: impl FnMut(&SfcStyleBlock<'_>, Option<&StyleObservation<'_>>),
    ) -> Result<VirtualTsOptions, SourceFrameError> {
        self.observe_styles::<true>(descriptor, strict, resolve_imports, visit)
    }

    fn observe_styles<const ALL: bool>(
        &self,
        descriptor: &SfcDescriptor<'_>,
        strict: bool,
        resolve_imports: bool,
        mut visit: impl FnMut(&SfcStyleBlock<'_>, Option<&StyleObservation<'_>>),
    ) -> Result<VirtualTsOptions, SourceFrameError> {
        let root = SourceRoot::new(&descriptor.source)?;
        let mut modules: BTreeMap<&str, Option<CssModuleShape>> = BTreeMap::new();
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
                visit(style, None);
                continue;
            };
            let original = vize_l1::container::lang::style_lang(style.lang.as_deref())
                .ok()
                .map(|language| StyleObservation::read(language, block));
            visit(style, original.as_ref());
            let shape =
                original.and_then(|observed| module_shape(style, observed, resolve_imports));
            match modules.entry(name) {
                Entry::Vacant(entry) => {
                    entry.insert(shape);
                }
                Entry::Occupied(mut entry) => match (entry.get_mut(), shape) {
                    (Some(previous), Some(shape)) => {
                        previous.classes.extend(shape.classes);
                        previous.imports.extend(shape.imports);
                    }
                    (previous, None) => *previous = None,
                    (None, Some(_)) => {}
                },
            }
        }
        let mut globals = self.template_globals.clone();
        let mut unknown = if modules.is_empty() {
            self.css_modules.clone()
        } else {
            Vec::new()
        };
        globals.retain(|global| !modules.contains_key(global.name.as_str()));
        for (name, shape) in modules {
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
