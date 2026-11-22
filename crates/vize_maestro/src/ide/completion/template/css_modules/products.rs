//! Checker options and complete CSS-module completion items for one open revision.
use super::{IdeContext, is_identifier_char};
use std::sync::Arc;
use tower_lsp::lsp_types::{
    CompletionItem, CompletionItemKind, Documentation, MarkupContent, MarkupKind,
};
use vize_canon::virtual_ts::VirtualTsOptions;
use vize_l0::{SmallVec, SourceFrameError, String as CompactString};

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
        self.read_style_module_products::<false>(base, strict, resolve_imports, |_, _| {})
    }

    fn read_style_module_products<const ALL: bool>(
        &self,
        base: &VirtualTsOptions,
        strict: bool,
        resolve_imports: bool,
        mut visit: impl FnMut(
            &vize_atelier_sfc::SfcStyleBlock<'_>,
            Option<&vize_l1::css::lang::StyleObservation<'_>>,
        ),
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
        let mut names: Option<SmallVec<[CompactString; 8]>> = None;
        let mut read =
            |style: &vize_atelier_sfc::SfcStyleBlock<'_>,
             observed: Option<&vize_l1::css::lang::StyleObservation<'_>>| {
                visit(style, observed);
                let Some(module) = style.module.as_deref() else {
                    return;
                };
                if Some(module) == receiver {
                    let names = names.get_or_insert_with(SmallVec::new);
                    if style.src.is_none()
                        && let Some(classes) = observed.and_then(|o| o.authored_classes())
                    {
                        names.extend(classes.names().iter().cloned());
                    }
                }
            };
        let checker = if ALL {
            base.observe_file_styles(descriptor, strict, resolve_imports, read)
        } else {
            base.observe_style_modules(descriptor, strict, resolve_imports, |_, s, o| read(s, o))
        };
        let completions = names.map(|mut names| {
            names.sort_unstable();
            names.dedup();
            let mut items = Vec::with_capacity(names.len());
            for name in names {
                items.push(class_completion(&name));
            }
            items
        });
        current()?;
        Ok(Some(StyleModuleToolProducts {
            checker: checker.map_err(StyleModuleRefusal::Frame)?,
            completions,
        }))
    }
}

fn module_receiver(before: &str) -> Option<&str> {
    let receiver = before
        .trim_end_matches(is_identifier_char)
        .strip_suffix('.')?;
    receiver
        .rsplit(|character| !is_identifier_char(character))
        .next()
        .filter(|module| !module.is_empty())
}

fn class_completion(name: &str) -> CompletionItem {
    #[expect(
        clippy::disallowed_macros,
        reason = "tower-lsp payload fields take std `String`, which `cstr!` does not produce"
    )]
    CompletionItem {
        label: name.to_string(),
        kind: Some(CompletionItemKind::FIELD),
        detail: Some("CSS module class".to_string()),
        documentation: Some(Documentation::MarkupContent(MarkupContent {
            kind: MarkupKind::Markdown,
            value: format!("`{name}` from `<style module>`."),
        })),
        sort_text: Some(format!("0{name}")),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests;

#[cfg(feature = "experimental-style-lint-products")]
mod lint;
