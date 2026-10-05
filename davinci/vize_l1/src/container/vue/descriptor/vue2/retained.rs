//! Complete original descriptor custody across the genuine expression handoff.

use super::super::{DescriptorIssue, DescriptorOptions, TemplateNamePair, TemplateSelection};
use super::{Vue2DescriptorObservation, Vue2DescriptorRefusal};
use crate::container::Container;
use crate::dialect::vue2::surface::ComponentExpressionPool;
use vize_l0::{SourceBlock, SourceRoot, Span, Vec};

/// The original complete envelope and Component, with owning expression slots.
/// Fields are private; no capture, replacement component or caller frame can
/// construct this pool. Its selected view still grants only envelope custody.
///
/// ```compile_fail
/// use vize_l1::container::vue::{Vue2DescriptorExpressionPool, Vue2DescriptorObservation};
/// fn clone(pool: Vue2DescriptorExpressionPool<'_>) { let _ = pool.clone(); }
/// ```
/// A public splitter capture cannot create an original expression pool:
/// ```compile_fail
/// use vize_l0::Allocator;
/// use vize_l1::container::{ContainerFormat, Vue, vue::Vue2DescriptorExpressionPool};
/// let arena = Allocator::default();
/// let pool: Vue2DescriptorExpressionPool<'_> = Vue.split(&arena, "<template/>").into();
/// ```
/// Nor can callers attach a detached body to a public capture/frame. Every
/// field here has its real type; private construction rejects this factory:
/// ```compile_fail
/// use vize_l0::{Allocator, SourceRoot, Vec, config::{VueDialect, VueVersion}};
/// use vize_l1::{SurfaceParseOptions, dialect::vue2::surface,
///     container::{ContainerFormat, Vue, vue::{DescriptorOptions, Vue2DescriptorExpressionPool}}};
/// let arena = Allocator::default();
/// let source = "<template>{{value}}</template>";
/// let foreign = surface::parse_component(&arena, "{{value}}").unwrap().into_expression_pool();
/// let _ = Vue2DescriptorExpressionPool {
///     container: Vue.split(&arena, source), root: SourceRoot::new(source).ok(),
///     options: DescriptorOptions { version: VueVersion::V2, dialect: VueDialect::Vue,
///         template: SurfaceParseOptions::default() },
///     issues: Vec::new_in(&arena), selection: None, component: Some(foreign), selected: true,
/// };
/// ```
/// A genuine root keeps its original source and allocator alive:
/// ```compile_fail
/// use vize_l0::{Allocator, String, config::{VueDialect, VueVersion}};
/// use vize_l1::{SurfaceParseOptions, container::{Vue, vue::DescriptorOptions}};
/// let arena = Allocator::default();
/// let root = {
///     let source = String::from("<template>{{value}}</template>");
///     let pool = Vue.observe_vue2_descriptor(&arena, &source, DescriptorOptions {
///         version: VueVersion::V2, dialect: VueDialect::Vue,
///         template: SurfaceParseOptions::default(),
///     }).into_expression_pool();
///     pool.component().unwrap().bindings().first().unwrap().admitted().unwrap()
///         .base().expression().unwrap()
/// };
/// println!("{root:?}");
/// ```
/// ```compile_fail
/// use vize_l0::{Allocator, config::{VueDialect, VueVersion}};
/// use vize_l1::{SurfaceParseOptions, container::{Vue, vue::DescriptorOptions}};
/// let root = {
///     let arena = Allocator::default();
///     let pool = Vue.observe_vue2_descriptor(&arena, "<template>{{value}}</template>",
///         DescriptorOptions { version: VueVersion::V2, dialect: VueDialect::Vue,
///             template: SurfaceParseOptions::default() }).into_expression_pool();
///     pool.component().unwrap().bindings().first().unwrap().admitted().unwrap()
///         .base().expression().unwrap()
/// };
/// println!("{root:?}");
/// ```
#[derive(Debug)]
pub struct Vue2DescriptorExpressionPool<'a> {
    #[cfg(test)]
    _drop_probe: super::hooks::OwnerDrop,
    container: Container<'a>,
    root: Option<SourceRoot<'a>>,
    options: DescriptorOptions,
    issues: Vec<'a, DescriptorIssue>,
    selection: Option<TemplateSelection<'a>>,
    component: Option<ComponentExpressionPool<'a>>,
    // Set only by the actual original selected() before consuming that owner.
    selected: bool,
}

impl<'a> Vue2DescriptorExpressionPool<'a> {
    pub fn source(&self) -> &'a str {
        self.container.source
    }
    pub fn root(&self) -> Option<SourceRoot<'a>> {
        self.root
    }
    pub fn options(&self) -> DescriptorOptions {
        self.options
    }
    pub fn container(&self) -> &Container<'a> {
        &self.container
    }
    pub fn issues(&self) -> &[DescriptorIssue] {
        &self.issues
    }
    pub fn component(&self) -> Option<&ComponentExpressionPool<'a>> {
        self.component.as_ref()
    }

    /// Borrow the same original selected frame and complete moved body.
    /// Body holes, callbacks, diagnostics and boundaries are still separate.
    pub fn selected(&self) -> Result<Vue2RetainedTemplateView<'_, 'a>, Vue2DescriptorRefusal<'_>> {
        let view = self
            .selected
            .then(|| {
                let selection = self.selection.as_ref()?;
                Some(Vue2RetainedTemplateView {
                    owner: self,
                    component: self.component.as_ref()?,
                    selection,
                    names: selection.names.as_ref()?,
                })
            })
            .flatten();
        view.ok_or(Vue2DescriptorRefusal {
            issues: &self.issues,
            errors: &self.container.errors,
        })
    }
}

impl<'a> Vue2DescriptorObservation<'a> {
    /// Consume this actual envelope and its complete original Component.
    /// Only the existing NativeSyntax consuming API relocates expression roots;
    /// nothing is reparsed, decoded again, cloned or silently omitted.
    /// Every refused slot preserves the returned original owner in this pool.
    /// All already retained and subsequent slots remain in original order.
    ///
    /// A root from the actual owning handoff survives ordinary pool drop:
    /// ```
    /// use vize_l0::{Allocator, config::{VueDialect, VueVersion}};
    /// use vize_l1::{SurfaceParseOptions, container::{Vue, vue::DescriptorOptions}};
    /// let arena = Allocator::default();
    /// let pool = Vue.observe_vue2_descriptor(&arena, "<template>{{value}}</template>",
    ///     DescriptorOptions { version: VueVersion::V2, dialect: VueDialect::Vue,
    ///         template: SurfaceParseOptions::default() }).into_expression_pool();
    /// let root = pool.component().unwrap().bindings().first().unwrap().admitted().unwrap()
    ///     .base().expression().unwrap();
    /// drop(pool);
    /// assert!(matches!(root, oxc_ast::ast::Expression::Identifier(_)));
    /// ```
    pub fn into_expression_pool(self) -> Vue2DescriptorExpressionPool<'a> {
        let selected = self.selected().is_ok();
        let Self {
            container,
            root,
            options,
            issues,
            selection,
            component,
            #[cfg(test)]
                _drop_probe: drop_probe,
        } = self;
        Vue2DescriptorExpressionPool {
            container,
            root,
            options,
            issues,
            selection,
            component: component.map(|component| component.into_expression_pool()),
            selected,
            #[cfg(test)]
            _drop_probe: drop_probe,
        }
    }
}

/// A sealed borrow of the actual moved envelope, body and stored selection.
/// No original CST-child, native File or runtime admission is implied.
///
/// ```compile_fail
/// use vize_l1::container::vue::Vue2DescriptorExpressionPool;
/// fn discard(pool: Vue2DescriptorExpressionPool<'_>) {
///     let view = pool.selected().unwrap();
///     drop(pool);
///     let _ = view.component();
/// }
/// ```
/// ```compile_fail
/// use vize_l1::container::vue::{Vue2DescriptorExpressionPool, Vue2RetainedTemplateView};
/// fn forge<'o, 'a>(pool: &'o Vue2DescriptorExpressionPool<'a>,
///     foreign: &'o Vue2DescriptorExpressionPool<'a>) {
///     let _ = Vue2RetainedTemplateView {
///         owner: pool, component: foreign.component().unwrap(), selection: todo!(), names: todo!(),
///     };
/// }
/// ```
#[derive(Debug)]
pub struct Vue2RetainedTemplateView<'o, 'a> {
    owner: &'o Vue2DescriptorExpressionPool<'a>,
    component: &'o ComponentExpressionPool<'a>,
    selection: &'o TemplateSelection<'a>,
    names: &'o TemplateNamePair,
}

impl<'o, 'a> Vue2RetainedTemplateView<'o, 'a> {
    pub fn observation(&self) -> &'o Vue2DescriptorExpressionPool<'a> {
        self.owner
    }
    pub fn component(&self) -> &'o ComponentExpressionPool<'a> {
        self.component
    }
    pub fn block(&self) -> SourceBlock<'a> {
        self.component.block()
    }
    pub fn container_index(&self) -> usize {
        self.selection.selection.index
    }
    pub fn opening_name(&self) -> Span {
        self.names.opening
    }
    pub fn closing_name(&self) -> Span {
        self.names.closing
    }
}

#[cfg(test)]
mod tests;
