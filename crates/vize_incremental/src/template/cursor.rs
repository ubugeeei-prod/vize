//! Reuse an exact immutable editor snapshot and its selected template.
#![expect(
    clippy::disallowed_types,
    reason = "the editor shares its existing std Arc<String> snapshot; a weak reference tracks identity"
)]
use super::{TemplateProducts, TemplateRead, TemplateReadError, TemplateRefusal};
use std::sync::{Arc, Weak};
use vize_l0::{
    SourceFrameError, SourceRoot, Span, String,
    config::{VueDialect, VueVersion},
    pass::PassObserver,
};
use vize_l1::{
    container::{Vue, vue::DescriptorOptions},
    markup::parse_with_options,
};
use vize_l1_to_l2::{
    lower::{LegacyCaps, LowerPolicy, lower_with_caps_and_comment_policy},
    pass::template::analyze,
};

struct CachedTemplate {
    source: String,
    content: Span,
    opening: Span,
    products: TemplateProducts,
}

/// The last admitted template of an open document. Each document can be read independently.
pub struct TemplateDocument {
    previous: Weak<std::string::String>,
    selected: Result<Option<CachedTemplate>, TemplateReadError>,
}

impl Default for TemplateDocument {
    fn default() -> Self {
        Self {
            previous: Weak::new(),
            selected: Ok(None),
        }
    }
}

impl core::fmt::Debug for TemplateDocument {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let range = self
            .selected
            .as_ref()
            .map(|entry| entry.as_ref().map(|entry| entry.content));
        f.debug_struct("TemplateDocument")
            .field("content", &range)
            .finish_non_exhaustive()
    }
}

impl TemplateDocument {
    /// Read both structural passes for this immutable Vue 3 snapshot using default markup options.
    /// The observer sees a group traversal only when a newly selected body needs analysis.
    pub fn read<'source, 'read>(
        &'read mut self,
        source: &'source Arc<std::string::String>,
        observer: &mut impl PassObserver,
    ) -> Result<Option<TemplateRead<'source, 'read>>, TemplateReadError> {
        // A weak reference reserves this allocation identity without retaining the old file bytes.
        if self.previous.as_ptr() != Arc::as_ptr(source) {
            if let Err(error) = self.refresh(source, observer) {
                self.selected = Err(error);
            }
            self.previous = Arc::downgrade(source);
        }
        let Some(entry) = self.selected.as_ref().map_err(Clone::clone)? else {
            return Ok(None);
        };
        let content = entry.content;
        let bytes = source
            .get(content.start as usize..content.end as usize)
            .ok_or(TemplateReadError::Source(SourceFrameError::BlockBoundary))?;
        let block = SourceRoot::new(source)
            .and_then(|root| root.block(bytes, content.start))
            .map_err(TemplateReadError::Source)?;
        Ok(Some(TemplateRead {
            block,
            opening: entry.opening,
            products: &entry.products,
        }))
    }

    fn refresh(
        &mut self,
        source: &str,
        observer: &mut impl PassObserver,
    ) -> Result<(), TemplateReadError> {
        let arena = vize_l0::pool::acquire();
        let descriptor = Vue.observe_descriptor(
            &arena,
            source,
            DescriptorOptions {
                version: VueVersion::V3,
                dialect: VueDialect::Vue,
                template: Default::default(),
            },
        );
        let Some(selected) = descriptor.admitted_template().map_err(|refusal| {
            TemplateReadError::Descriptor(Arc::new(TemplateRefusal {
                issues: refusal.issues().into(),
                errors: refusal.errors().into(),
            }))
        })?
        else {
            self.selected = Ok(None);
            return Ok(());
        };
        let block = selected.block();
        let source = block.source();
        // Descriptor, L1 body and lowering share an arena on a content edit.
        let mut compute = || {
            let (tree, errors) = parse_with_options(&arena, source, selected.surface_options());
            let mut lowered = lower_with_caps_and_comment_policy::<vize_l2::provenance::NoProvenance>(
                &arena,
                &tree,
                &errors,
                LegacyCaps::VUE3,
                LowerPolicy::compile(false),
                &[],
                None,
            );
            let analysis = analyze(&mut lowered, observer).map_err(TemplateReadError::Pass)?;
            Ok::<_, TemplateReadError>(TemplateProducts {
                analysis,
                diagnostics: lowered.diagnostics,
            })
        };
        if let Ok(Some(entry)) = &mut self.selected {
            if entry.source.as_str() != source {
                let products = compute()?;
                entry.source.clear();
                entry.source.push_str(source);
                entry.products = products;
            }
            entry.content = block.span();
            entry.opening = selected.open_tag();
        } else {
            self.selected = Ok(Some(CachedTemplate {
                source: String::from(source),
                content: block.span(),
                opening: selected.open_tag(),
                products: compute()?,
            }));
        }
        Ok(())
    }
}
