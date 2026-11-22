//! CSS expressions that the checker considers used outside the template.
use vize_carton::{FxHashSet, String};
use vize_l0::{Allocator, SmallVec, SourceRoot};
use vize_l1::container::vue::AdmittedDescriptor;
use vize_l1_to_l2::lower::OriginalStyleBindings;
use vize_l2::expr::ExprRef;
use vize_relief::JsExpression;

/// The shipped file parser's refusal, or incompatible authored style selection.
#[derive(Debug)]
pub enum StyleBindingInputError {
    Descriptor(vize_croquis::sfc::SfcError),
    StyleSelection,
}

/// The checker's CSS references and the compiler descriptor's unique expressions,
/// in first-occurrence order across all the original style blocks supplied.
#[derive(Default)]
pub struct StyleBindingReferences<'a> {
    expressions: SmallVec<[&'a str; 4]>,
    names: FxHashSet<String>,
}

impl<'a> StyleBindingReferences<'a> {
    /// Prepare the real compiler descriptor and checker references from one read
    /// of its CSS calls. Preserve the compiler's parser errors before publishing
    /// any style. Each tool consumes the same original content and byte range.
    pub fn prepare(
        allocator: &'a Allocator,
        source: &'a str,
        options: vize_croquis::sfc::SfcParseOptions,
        visit: impl FnMut(OriginalStyleBindings<'a>),
    ) -> Result<(vize_croquis::sfc::SfcDescriptor<'a>, Self), StyleBindingInputError> {
        // The compiler already has a validated fast parser for a lone template.
        // A nested SVG style or an import containing "<style" is part of that
        // template. Check the outer spelling without scanning the file body.
        let possible_lone_template = source.trim_ascii_start().starts_with("<template")
            && source.trim_ascii_end().ends_with("</template>");
        let mut descriptor = if possible_lone_template {
            vize_atelier_sfc::parse_sfc(source, options)
        } else {
            vize_croquis::sfc::parse_sfc_without_css_vars(source, options)
        }
        .map_err(StyleBindingInputError::Descriptor)?;
        if descriptor.styles.is_empty() {
            return Ok((descriptor, Self::default()));
        }
        let root = SourceRoot::new(source).map_err(|_| StyleBindingInputError::StyleSelection)?;
        // Check every original slice before publishing any style to another tool.
        let original = |style: &vize_croquis::sfc::SfcStyleBlock<'_>| {
            let start = u32::try_from(style.loc.start).ok()?;
            let content = source.get(style.loc.start..style.loc.end)?;
            if !core::ptr::eq(content, style.content.as_ref()) && content != style.content {
                return None;
            }
            root.block(content, start).ok()
        };
        if !descriptor
            .styles
            .iter()
            .all(|style| original(style).is_some())
        {
            return Err(StyleBindingInputError::StyleSelection);
        }
        let mut facts = Self::default();
        let mut visit = visit;
        for block in descriptor.styles.iter().filter_map(original) {
            let style = OriginalStyleBindings::from_block(allocator, block);
            facts.extend(&style);
            visit(style);
        }
        // A multi-block file may also start and end with template spelling.
        descriptor.css_vars.clear();
        descriptor.css_vars.extend(
            facts
                .expressions
                .iter()
                .copied()
                .map(std::borrow::Cow::Borrowed),
        );
        Ok((descriptor, facts))
    }

    /// Lower the original file's styles in descriptor order and expose each
    /// carrier to another consumer before returning the complete checker input.
    /// A file without a style block has no work for this pass and returns `None`.
    pub fn observe(
        allocator: &'a Allocator,
        descriptor: AdmittedDescriptor<'_, 'a>,
        mut visit: impl FnMut(OriginalStyleBindings<'a>),
    ) -> Option<Self> {
        let styles = descriptor.styles();
        if styles.len() == 0 {
            return None;
        }
        let mut result = Self::default();
        for original in styles {
            let style = OriginalStyleBindings::read(allocator, original);
            result.extend(&style);
            visit(style);
        }
        Some(result)
    }

    fn extend(&mut self, style: &OriginalStyleBindings<'a>) {
        for call in style.calls() {
            let source = call.value.source();
            if self.expressions.contains(&source) {
                continue;
            }
            self.expressions.push(source);
            let retained = match call.value {
                ExprRef::Js(js) => Some(JsExpression {
                    ast: js.ast,
                    raw: js.source,
                }),
                _ => None,
            };
            self.names
                .extend(vize_croquis::drawer::extract_identifiers_retained(
                    source,
                    retained.as_ref(),
                ));
        }
    }

    /// Generate a setup projection with unused-local checking enabled. CSS-only
    /// setup reads enter the same anchor selection as the template and script.
    #[must_use]
    pub fn generate_setup_virtual_ts(
        &self,
        summary: &vize_croquis::Croquis,
        script: Option<&str>,
        template: Option<&vize_relief::RootNode<'_>>,
        script_offset: u32,
        template_offset: u32,
        options: &super::VirtualTsOptions,
    ) -> super::VirtualTsOutput {
        super::generate_virtual_ts_with_offsets_and_checks(
            summary,
            script,
            template,
            script_offset,
            template_offset,
            options,
            super::VirtualTsGenerationOptions {
                preserve_unused_diagnostics: true,
                extra_template_referenced_names: Some(&self.names),
                ..Default::default()
            },
        )
    }

    #[must_use]
    pub fn expressions(&self) -> &[&'a str] {
        &self.expressions
    }

    #[must_use]
    pub fn names(&self) -> &FxHashSet<String> {
        &self.names
    }
    /// Transfer the finished CSS read set into a request's other reference inputs.
    #[must_use]
    pub fn into_names(self) -> FxHashSet<String> {
        self.names
    }
}
