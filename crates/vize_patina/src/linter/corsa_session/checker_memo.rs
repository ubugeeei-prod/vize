//! Checker answers shared by every fact family of one expression batch.
//!
//! Promise, detail, shape, type-node and relation facts ask the checker the
//! same questions about the same types (union constituents, base constraints,
//! contextual types). One memo per batch answers each question once per type
//! or node, whichever family asks first.
use corsa::api::{NodeHandle, ProjectSession, TypeHandle, TypeResponse};
use std::cell::RefCell;
use std::ops::Deref;
use vize_l0::FxHashMap;

/// A [`ProjectSession`] with per-batch memoized checker answers.
///
/// It derefs to the session, so uncached requests read as before.
pub(in crate::linter) struct CheckerMemo<'a> {
    session: &'a ProjectSession,
    constituents: RefCell<FxHashMap<TypeHandle, Vec<TypeResponse>>>,
    base_constraints: RefCell<FxHashMap<TypeHandle, Option<TypeResponse>>>,
    contextual: RefCell<FxHashMap<NodeHandle, Option<TypeResponse>>>,
    texts: RefCell<FxHashMap<TypeHandle, Vec<vize_l0::String>>>,
    property_names: RefCell<FxHashMap<TypeHandle, Vec<vize_l0::String>>>,
}

impl Deref for CheckerMemo<'_> {
    type Target = ProjectSession;

    fn deref(&self) -> &ProjectSession {
        self.session
    }
}

impl<'a> CheckerMemo<'a> {
    pub(in crate::linter) fn new(session: &'a ProjectSession) -> Self {
        Self {
            session,
            constituents: RefCell::default(),
            base_constraints: RefCell::default(),
            contextual: RefCell::default(),
            texts: RefCell::default(),
            property_names: RefCell::default(),
        }
    }

    /// The session itself, for helpers that take `&ProjectSession`.
    pub(in crate::linter) fn session(&self) -> &'a ProjectSession {
        self.session
    }

    /// `getTypesOfType`: union or intersection constituents (empty for other
    /// types, or when the checker cannot answer).
    pub(in crate::linter) async fn constituents(&self, ty: &TypeResponse) -> Vec<TypeResponse> {
        if let Some(found) = self.constituents.borrow().get(&ty.id) {
            return found.clone();
        }
        let found = self
            .session
            .client()
            .get_types_of_type_in_project(
                self.session.snapshot().handle.clone(),
                self.session.project_handle(),
                ty.id.clone(),
            )
            .await
            .unwrap_or_default();
        self.constituents
            .borrow_mut()
            .insert(ty.id.clone(), found.clone());
        found
    }

    /// `getBaseConstraintOfType`. `None` when the checker has none.
    pub(in crate::linter) async fn base_constraint(
        &self,
        ty: &TypeResponse,
    ) -> Option<TypeResponse> {
        if let Some(found) = self.base_constraints.borrow().get(&ty.id) {
            return found.clone();
        }
        let found = self
            .session
            .client()
            .get_base_constraint_of_type(
                self.session.snapshot().handle.clone(),
                self.session.project_handle(),
                ty.id.clone(),
            )
            .await
            .ok()
            .flatten();
        self.base_constraints
            .borrow_mut()
            .insert(ty.id.clone(), found.clone());
        found
    }

    /// The checker's contextual type at `location`, if any.
    pub(in crate::linter) async fn contextual_type(
        &self,
        location: &NodeHandle,
    ) -> Option<TypeResponse> {
        if let Some(found) = self.contextual.borrow().get(location) {
            return found.clone();
        }
        let found = self
            .session
            .client()
            .get_contextual_type(
                self.session.snapshot().handle.clone(),
                self.session.project_handle(),
                location.clone(),
            )
            .await
            .ok()
            .flatten();
        self.contextual
            .borrow_mut()
            .insert(location.clone(), found.clone());
        found
    }

    /// Rendered type texts (the response's own texts, else `typeToString`).
    pub(in crate::linter) async fn type_texts(&self, ty: &TypeResponse) -> Vec<vize_l0::String> {
        if let Some(found) = self.texts.borrow().get(&ty.id) {
            return found.clone();
        }
        let found = self
            .session
            .render_type_texts(ty)
            .await
            .unwrap_or_default()
            .into_iter()
            .map(|text| vize_l0::String::from(text.as_str()))
            .collect::<Vec<_>>();
        self.texts.borrow_mut().insert(ty.id.clone(), found.clone());
        found
    }

    /// Names of the properties visible on `ty`.
    pub(in crate::linter) async fn property_names(
        &self,
        ty: &TypeResponse,
    ) -> Vec<vize_l0::String> {
        if let Some(found) = self.property_names.borrow().get(&ty.id) {
            return found.clone();
        }
        let found = self
            .session
            .get_properties_of_type(ty.id.clone())
            .await
            .unwrap_or_default()
            .into_iter()
            .map(|property| vize_l0::String::from(property.name.as_str()))
            .collect::<Vec<_>>();
        self.property_names
            .borrow_mut()
            .insert(ty.id.clone(), found.clone());
        found
    }
}
