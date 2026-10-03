//! Descriptor-selected original template ownership, without File completion.

use super::{NativeChildren, NativeComponent};
use crate::container::vue::{AdmittedDescriptor, ScriptRole, ScriptView};
use crate::{embed::Lang, markup::ComponentSourceError};
use vize_l0::{Allocator, SourceBlock};

/// Intrinsic first-family grammar, selected by the authentic Descriptor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeTemplateGrammar {
    JavaScriptModule,
    TypeScriptModule,
}

#[derive(Debug, Clone, Copy)]
struct Selection<'a> {
    index: usize,
    block: SourceBlock<'a>,
    lang: Lang,
    role: ScriptRole,
}
impl<'a> Selection<'a> {
    fn original(view: ScriptView<'_, 'a>) -> Self {
        Self {
            index: view.container_index(),
            block: view.block(),
            lang: view.lang(),
            role: view.role(),
        }
    }
}

/// Original Component parsed once from an authentic selected template.
/// The short Descriptor borrow ends before this owner is stored or moved.
/// No caller-selected block, language, role or raw Component can mint it.
/// This owner does not certify clean grammar or a completed native File.
///
/// ```compile_fail
/// use vize_l1::markup::NativeTemplateComponent;
/// fn requires_clone<T: Clone>() {}
/// requires_clone::<NativeTemplateComponent<'static>>();
/// ```
#[derive(Debug)]
pub struct NativeTemplateComponent<'a> {
    component: NativeComponent<'a>,
    index: usize,
    grammar: NativeTemplateGrammar,
    ordinary: Option<Selection<'a>>,
    setup: Option<Selection<'a>>,
}

impl<'a> NativeTemplateComponent<'a> {
    /// Missing selected template returns `None` without parsing another block.
    pub fn parse_in(
        allocator: &'a Allocator,
        descriptor: AdmittedDescriptor<'_, 'a>,
    ) -> Result<Option<Self>, ComponentSourceError> {
        let Some(template) = descriptor.template() else {
            return Ok(None);
        };
        let grammar = match descriptor.template_lang() {
            Lang::Js => NativeTemplateGrammar::JavaScriptModule,
            Lang::Ts => NativeTemplateGrammar::TypeScriptModule,
        };
        Ok(Some(Self {
            component: NativeComponent::parse_in(allocator, template.block())?,
            index: template.container_index(),
            grammar,
            ordinary: descriptor.ordinary().map(Selection::original),
            setup: descriptor.setup().map(Selection::original),
        }))
    }
    #[must_use]
    pub fn component(&self) -> &NativeComponent<'a> {
        &self.component
    }
    #[must_use]
    pub fn template_index(&self) -> usize {
        self.index
    }
    #[must_use]
    pub fn grammar(&self) -> NativeTemplateGrammar {
        self.grammar
    }
    #[must_use]
    pub fn children(&self) -> NativeChildren<'_, 'a> {
        self.component.children()
    }
    #[must_use]
    pub fn ordinary(&self) -> Option<NativeScriptSelection<'_, 'a>> {
        self.ordinary.map(|selected| NativeScriptSelection {
            owner: self,
            selected,
        })
    }
    #[must_use]
    pub fn setup(&self) -> Option<NativeScriptSelection<'_, 'a>> {
        self.setup.map(|selected| NativeScriptSelection {
            owner: self,
            selected,
        })
    }
    /// Consuming transfer drops Descriptor selection authority.
    #[must_use]
    pub fn into_component(self) -> NativeComponent<'a> {
        self.component
    }
}

/// Readonly original script selection borrowing its selected template owner.
#[derive(Debug)]
pub struct NativeScriptSelection<'o, 'a> {
    owner: &'o NativeTemplateComponent<'a>,
    selected: Selection<'a>,
}
impl<'o, 'a> NativeScriptSelection<'o, 'a> {
    #[must_use]
    pub fn owner(&self) -> &'o NativeTemplateComponent<'a> {
        self.owner
    }
    #[must_use]
    pub fn container_index(&self) -> usize {
        self.selected.index
    }
    #[must_use]
    pub fn block(&self) -> SourceBlock<'a> {
        self.selected.block
    }
    #[must_use]
    pub fn lang(&self) -> Lang {
        self.selected.lang
    }
    #[must_use]
    pub fn role(&self) -> ScriptRole {
        self.selected.role
    }
}

#[cfg(test)]
mod tests;
