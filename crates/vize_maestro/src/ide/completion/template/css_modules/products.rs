//! Checker options and complete CSS-module completion items for one open revision.
use super::{IdeContext, class_completion, module_receiver};
use std::{collections::BTreeSet, sync::Arc};
use tower_lsp::lsp_types::CompletionItem;
use vize_canon::virtual_ts::VirtualTsOptions;
use vize_l0::SourceFrameError;

#[derive(Debug)]
pub struct StyleModuleToolProducts {
    pub checker: VirtualTsOptions,
    pub completions: Option<Vec<CompletionItem>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StyleModuleRefusal {
    Closed,
    Superseded,
    Frame(SourceFrameError),
}

impl IdeContext<'_> {
    /// Read checker and completion products against the same open Vue source.
    pub fn style_module_products(
        &self,
        base: &VirtualTsOptions,
        strict: bool,
        resolve_imports: bool,
    ) -> Result<Option<StyleModuleToolProducts>, StyleModuleRefusal> {
        let current = || {
            let text = self
                .state
                .documents
                .text_snapshot(self.uri)
                .ok_or(StyleModuleRefusal::Closed)?;
            if !Arc::ptr_eq(&text, &self.content) {
                return Err(StyleModuleRefusal::Superseded);
            }
            Ok(())
        };
        current()?;
        let Some(descriptor) = self.descriptor() else {
            current()?;
            return Ok(None);
        };
        let receiver = self
            .content
            .get(..self.offset.min(self.content.len()))
            .and_then(module_receiver);
        let mut names = None;
        let checker = base.observe_style_modules(
            descriptor,
            strict,
            resolve_imports,
            |module, style, observed| {
                if Some(module) == receiver {
                    let names = names.get_or_insert_with(BTreeSet::new);
                    if style.src.is_none()
                        && let Some(classes) = observed.and_then(|o| o.authored_classes())
                    {
                        names.extend(classes.names().iter().cloned());
                    }
                }
            },
        );
        let completions = names.map(|names| {
            names
                .into_iter()
                .map(|name| class_completion(&name))
                .collect()
        });
        current()?;
        Ok(Some(StyleModuleToolProducts {
            checker: checker.map_err(StyleModuleRefusal::Frame)?,
            completions,
        }))
    }
}

#[cfg(test)]
mod tests;
