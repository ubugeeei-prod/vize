//! Store template row identities without self-references. Rehydrating a row
//! borrows the same immutable owner whose original sites were checked.

use super::{
    FileArtifact, HandlerLocalRef, Origin, ScopeId, Span, TemplateQueryError, TemplateSiteRef,
    TemplateSiteScope, TemplateSymbolRef, Usage, sites,
};
use crate::op::{HandlerId, OriginalForId};
use crate::query::position::index::SiteIntervals;
use crate::resolution::{BindingId, HandlerLocalId, HandlerScopeId};

enum Site {
    Handler {
        owner: HandlerId,
        binding: HandlerBinding,
        scope: HandlerScopeId,
    },
    For {
        owner: OriginalForId,
        binding: BindingId,
        scope: ScopeId,
    },
}

enum HandlerBinding {
    Local(HandlerLocalId),
    Outer(BindingId),
}

struct StoredSite {
    site: Site,
    usage: Option<Usage>,
}

impl StoredSite {
    fn capture(site: TemplateSiteRef<'_, '_>) -> Result<Self, TemplateQueryError> {
        let record = match (site.origin, site.scope, site.symbol) {
            (Origin::Handler(owner), TemplateSiteScope::Handler(_, scope), symbol) => {
                Site::Handler {
                    owner: owner.id(),
                    scope,
                    binding: match symbol {
                        TemplateSymbolRef::File(binding) => HandlerBinding::Outer(binding.id()),
                        TemplateSymbolRef::HandlerLocal(local) => {
                            HandlerBinding::Local(local.binding.id)
                        }
                    },
                }
            }
            (
                Origin::For(owner),
                TemplateSiteScope::File(scope),
                TemplateSymbolRef::File(binding),
            ) => Site::For {
                owner: owner.id(),
                scope,
                binding: binding.id(),
            },
            _ => return Err(TemplateQueryError::OriginalMembership),
        };
        Ok(Self {
            site: record,
            usage: site.usage,
        })
    }

    fn restore<'f, 'a>(
        &self,
        file: &'f FileArtifact<'a>,
        span: Span,
    ) -> Option<TemplateSiteRef<'f, 'a>> {
        let (origin, scope, symbol) = match self.site {
            Site::Handler {
                owner,
                ref binding,
                scope,
            } => {
                let handler = file.handler(owner)?;
                let symbol = match binding {
                    HandlerBinding::Outer(id) => TemplateSymbolRef::File(file.binding(*id)?),
                    HandlerBinding::Local(id) => {
                        let binding = handler.resolution()?.bindings().get(id.index() as usize)?;
                        TemplateSymbolRef::HandlerLocal(HandlerLocalRef { handler, binding })
                    }
                };
                (
                    Origin::Handler(handler),
                    TemplateSiteScope::Handler(handler, scope),
                    symbol,
                )
            }
            Site::For {
                owner,
                binding,
                scope,
            } => (
                Origin::For(file.for_head(owner)?),
                TemplateSiteScope::File(scope),
                TemplateSymbolRef::File(file.binding(binding)?),
            ),
        };
        Some(TemplateSiteRef {
            origin,
            scope,
            symbol,
            span,
            usage: self.usage,
        })
    }
}

pub(in crate::query) struct TemplatePositions(
    Result<SiteIntervals<StoredSite>, TemplateQueryError>,
);

impl Default for TemplatePositions {
    fn default() -> Self {
        Self(Ok(SiteIntervals::default()))
    }
}

impl TemplatePositions {
    pub(in crate::query) fn new(file: &FileArtifact<'_>) -> Self {
        Self(SiteIntervals::try_from_sites(|push| {
            let mut captured = Ok(());
            sites::visit(file, |site| match StoredSite::capture(site) {
                Ok(record) => push(site.span, record),
                Err(error) => captured = Err(error),
            })?;
            captured
        }))
    }

    pub(in crate::query) fn at<'f, 'a>(
        &self,
        file: &'f FileArtifact<'a>,
        offset: u32,
    ) -> Result<Option<TemplateSiteRef<'f, 'a>>, TemplateQueryError> {
        let index = self.0.as_ref().map_err(|e| *e)?;
        let selected = index
            .unique(offset, |span, record| Some((span, record)))
            .map_err(TemplateQueryError::Position)?;
        selected
            .map(|(span, record)| {
                record
                    .restore(file, span)
                    .ok_or(TemplateQueryError::OriginalMembership)
            })
            .transpose()
    }
}
