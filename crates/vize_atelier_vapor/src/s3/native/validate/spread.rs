//! Element objects: a `v-bind` object merges the element's static attributes
//! and `:prop`s in authored order (upstream's `setDynamicProps` sources), and
//! a `v-on` object binds its listeners with `setDynamicEvents`.
//!
//! Beside an object, only plain `:prop`s and static attributes are admitted.
//! Named listeners interleave with an object's listeners in an order the
//! Vapor and VDOM runtimes do not agree on, and directives (`v-show`,
//! `v-model`, content directives) keep their own runtime contracts.

use super::super::{BindingKind, Content, Node};
use super::Result;
use crate::s3::LegacyReason;

pub(super) fn check(nodes: &[Node<'_>]) -> Result<()> {
    for node in nodes {
        let Content::Element { tag, .. } = node.content else {
            continue;
        };
        let object = |kind| node.bindings.iter().any(|binding| binding.kind == kind);
        let (spread, handlers) = (object(BindingKind::Spread), object(BindingKind::Handlers));
        if (spread || handlers)
            && (tag == "template"
                || spread && handlers
                || (node.bindings.iter()).any(|binding| {
                    !matches!(
                        binding.kind,
                        BindingKind::Prop | BindingKind::Spread | BindingKind::Handlers
                    )
                }))
        {
            return Err(LegacyReason::Binding.into());
        }
    }
    Ok(())
}
