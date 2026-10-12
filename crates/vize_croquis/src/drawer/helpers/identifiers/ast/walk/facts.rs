//! Completeness carried by the existing identifier traversal, never a second walk.
use super::super::super::IdentifierRef;

pub(in super::super) struct IdentifierWalk {
    pub(in super::super) values: Vec<IdentifierRef>,
    pub(in super::super) demanded: bool,
    pub(in super::super) complete: bool,
    pub(in super::super) runtime_capture: bool,
    pub(in super::super) runtime_only: bool,
}

impl IdentifierWalk {
    pub(in super::super) fn new(demanded: bool) -> Self {
        Self {
            values: Vec::with_capacity(4),
            demanded,
            complete: true,
            runtime_capture: false,
            runtime_only: true,
        }
    }
    pub(super) fn child(&self) -> Self {
        let mut child = Self::new(self.demanded);
        child.runtime_capture = self.runtime_capture;
        child
    }
    pub(super) fn refuse_runtime_read(&mut self) {
        if self.runtime_capture {
            self.runtime_only = false;
        }
    }
    pub(super) fn refuse(&mut self) {
        if self.demanded {
            self.complete = false;
        }
    }
}
impl std::ops::Deref for IdentifierWalk {
    type Target = Vec<IdentifierRef>;
    fn deref(&self) -> &Self::Target {
        &self.values
    }
}
impl std::ops::DerefMut for IdentifierWalk {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.values
    }
}
