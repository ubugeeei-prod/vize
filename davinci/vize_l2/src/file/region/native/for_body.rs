//! Introducing node, authentic alias scope, then one original Element body.

use super::{NativeVisibility, body, for_head::ObservedFor};
use crate::artifact::RegionBuilder;
use crate::file::for_head::ForRecord;
use crate::file::region::FileRegion;
use crate::lang::js::file::native::NativeTemplateIssueKind as Kind;
use crate::op::OriginalForId;
use crate::resolution::ForAliasRole;
use core::{marker::PhantomData, ops::DerefMut};
use vize_l0::{Span, id::NodeId};
use vize_l1::markup::{NativeElement, NativeTemplateComponent};

pub(super) fn construct<'a: 'b, 'b, R>(
    original: NativeElement<'_, 'a>,
    selected: &NativeTemplateComponent<'a>,
    span: Span,
    header: body::header::Header<'a>,
    observed: ObservedFor,
    region: &mut FileRegion<'_, 'b, 'a, R, NativeVisibility>,
) -> Result<NodeId, Kind>
where
    R: DerefMut<Target = RegionBuilder<'b, 'a>>,
{
    region.resolve_for(&observed)?;
    region.with_walk(span, |file| {
        let pending = file
            .facts
            .pending_for_heads
            .get(observed.index)
            .ok_or(Kind::InvalidEvent)?;
        let head = pending.resolution.as_ref().ok_or(Kind::InvalidEvent)?;
        let introduction = head.input().operand().syntax().source().span();
        // End the normal owner borrow only after its checked introducing mint.
        // All subsequent scope/storage/callback work still has that owner parked.
        let node = file
            .region
            .begin_original_for(head, span)
            .map_err(Kind::Artifact)?;
        let id = OriginalForId(node);
        let scope = file
            .facts
            .template_scope(id, file.scope, introduction)
            .map_err(|kind| file.for_error(introduction, kind))?;
        let pending = file
            .facts
            .pending_for_heads
            .get_mut(observed.index)
            .ok_or(Kind::InvalidEvent)?;
        let resolution = pending.resolution.take().ok_or(Kind::InvalidEvent)?;
        let has_key = resolution.key().is_some();
        let _ = file.facts.for_heads.insert(
            node,
            ForRecord {
                original: None,
                enclosing: file.scope,
                scope,
                resolution,
                value: None,
                key: None,
            },
        );
        file.facts
            .declare_template_alias(id, ForAliasRole::Value)
            .map_err(|kind| file.for_error(introduction, kind))?;
        if has_key {
            file.facts
                .declare_template_alias(id, ForAliasRole::Key)
                .map_err(|kind| file.for_error(introduction, kind))?;
        }
        #[cfg(test)]
        super::for_tests::after_scope();
        {
            // Reborrow only the actual current factory. The parent scope is never
            // overwritten, including when a real header/body callback unwinds.
            let mut child = FileRegion {
                region: &mut *file.region,
                facts: &mut *file.facts,
                scope,
                policy: file.policy,
                source: file.source,
                borrow: PhantomData,
            };
            let ready = header.resolve_handlers(&mut child)?;
            let _ = body::element_ready(original, selected, span, ready, &mut child)?;
        }
        let pointer = file
            .region
            .finish_original_for(node)
            .map_err(Kind::Artifact)?;
        file.facts
            .for_heads
            .get_mut(node)
            .ok_or(Kind::InvalidEvent)?
            .original = Some(pointer);
        Ok(node)
    })
}
