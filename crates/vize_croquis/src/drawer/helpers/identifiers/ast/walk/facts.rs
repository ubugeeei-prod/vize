//! Completeness carried by the existing identifier traversal, never a second walk.
use super::super::super::IdentifierRef;

pub(in super::super) struct IdentifierWalk {
    pub(in super::super) values: Vec<IdentifierRef>,
    pub(in super::super) demanded: bool,
    pub(in super::super) complete: bool,
}

impl IdentifierWalk {
    pub(in super::super) fn new(demanded: bool) -> Self {
        Self {
            values: Vec::with_capacity(4),
            demanded,
            complete: true,
        }
    }
    pub(super) fn child(&self) -> Self {
        Self::new(self.demanded)
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
