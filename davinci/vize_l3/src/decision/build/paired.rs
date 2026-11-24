//! Compute the columns read by the client and server emitters on one owner.
use super::{
    Builder, DecisionAnalysis, DecisionBuildError, DomBuilder, LiteralExpressions, NoReads,
    OriginalAttributeCursor, SideTable, SsrBuilder, TargetPolicy,
};
use crate::decision::{
    OriginalAttributeFacts,
    policy::BindingOwner,
    ssr::{SsrFacts, SsrTemplateBuildError},
    template::{TemplateDomAnalysis, build_template_dom_decisions},
};
use alloc::vec::Vec;
use vize_l0::{Span, id::NodeId};
use vize_l2::{
    file::L2Ir,
    lang::js::{NativeTemplateFile, NativeTemplateIssue, NativeTemplateView},
    op::Op,
    walk::{NodeEvent, NodeRef},
};

/// Original SSR render facts and the selected File that owns their nodes and attributes.
pub struct ServerRenderAnalysis<'owner, 'arena> {
    owner: &'owner NativeTemplateFile<'arena>,
    file: &'owner L2Ir<'arena>,
    attributes: OriginalAttributeFacts<'owner, 'arena>,
    facts: SsrFacts<'owner, 'arena>,
}
impl<'owner, 'arena> ServerRenderAnalysis<'owner, 'arena> {
    pub fn owner(&self) -> &'owner NativeTemplateFile<'arena> {
        self.owner
    }
    pub fn file(&self) -> &'owner L2Ir<'arena> {
        self.file
    }
    pub fn original_attributes(&self) -> &OriginalAttributeFacts<'owner, 'arena> {
        &self.attributes
    }
    pub fn facts(&self) -> &SsrFacts<'owner, 'arena> {
        &self.facts
    }
}

/// Independent results for a client/server build of one Descriptor-selected owner.
pub struct ClientServerDecisions<'owner, 'arena> {
    dom: Result<TemplateDomAnalysis<'owner, 'arena>, DecisionBuildError>,
    ssr: Result<ServerRenderAnalysis<'owner, 'arena>, SsrTemplateBuildError>,
}
impl<'owner, 'arena> ClientServerDecisions<'owner, 'arena> {
    pub fn dom(&self) -> Result<&TemplateDomAnalysis<'owner, 'arena>, DecisionBuildError> {
        self.dom.as_ref().map_err(|error| *error)
    }
    pub fn ssr(&self) -> Result<&ServerRenderAnalysis<'owner, 'arena>, SsrTemplateBuildError> {
        self.ssr.as_ref().map_err(|error| *error)
    }
    fn refused(error: DecisionBuildError) -> Self {
        Self {
            dom: Err(error),
            ssr: Err(SsrTemplateBuildError::Decision(error)),
        }
    }
}

// Only this producer can associate a completed DOM table with the selected view.
pub(in crate::decision) struct CompletedTarget<'owner, 'arena> {
    view: NativeTemplateView<'owner, 'arena>,
    file: &'owner L2Ir<'arena>,
    analysis: DecisionAnalysis<'owner, 'arena>,
}
impl<'owner, 'arena> CompletedTarget<'owner, 'arena> {
    pub(in crate::decision) fn into_parts(
        self,
    ) -> (
        NativeTemplateView<'owner, 'arena>,
        &'owner L2Ir<'arena>,
        DecisionAnalysis<'owner, 'arena>,
    ) {
        (self.view, self.file, self.analysis)
    }
}

struct RenderPass<'owner, 'arena> {
    server: SsrBuilder<'owner, 'owner, 'arena>,
    attributes: Option<OriginalAttributeCursor<'owner, 'arena>>,
    binding_owner: Option<(NodeId, BindingOwner)>,
}
impl<'owner, 'arena> RenderPass<'owner, 'arena> {
    fn new(file: &'owner L2Ir<'arena>) -> Result<Self, DecisionBuildError> {
        Ok(Self {
            server: SsrBuilder::new(),
            attributes: Some(OriginalAttributeCursor::new(file, file.artifact())?),
            binding_owner: None,
        })
    }

    fn finish(
        self,
        owner: &'owner NativeTemplateFile<'arena>,
        file: &'owner L2Ir<'arena>,
    ) -> Result<ServerRenderAnalysis<'owner, 'arena>, DecisionBuildError> {
        let attributes = self
            .attributes
            .ok_or(DecisionBuildError::IncompleteFile)?
            .finish()?;
        Ok(ServerRenderAnalysis {
            owner,
            file,
            attributes,
            facts: self.server.finish(),
        })
    }

    fn visit(&mut self, event: NodeEvent<'owner, 'arena>) -> Result<(), DecisionBuildError> {
        match event {
            NodeEvent::Enter {
                id,
                node: NodeRef::Op(op),
                parent,
                ..
            } => {
                // Attached bindings immediately follow their element, component or slot.
                self.binding_owner = match op {
                    Op::Element(_) => Some((id, BindingOwner::Element)),
                    Op::Component(_) => Some((id, BindingOwner::Component)),
                    Op::Slot(_) => Some((id, BindingOwner::Slot)),
                    _ => None,
                };
                self.server
                    .enter(id, op, parent.is_none(), &mut self.attributes);
            }
            NodeEvent::Enter {
                id,
                node: NodeRef::Binding(binding),
                parent,
                ..
            } => {
                let Some((_, owner)) = self.binding_owner.filter(|(node, _)| Some(*node) == parent)
                else {
                    return Err(DecisionBuildError::InvalidTraversal { node: id });
                };
                self.server.binding(id, binding, owner);
            }
            NodeEvent::Leave {
                id,
                node: NodeRef::Op(_),
                ..
            } => self.server.leave(id),
            NodeEvent::Leave {
                node: NodeRef::Binding(_),
                ..
            } => {}
        }
        Ok(())
    }
}

type ClientPass<'owner, 'arena> = Builder<'owner, 'owner, 'arena, LiteralExpressions, NoReads>;

fn client_pass<'owner, 'arena>(
    file: &'owner L2Ir<'arena>,
) -> Result<ClientPass<'owner, 'arena>, DecisionBuildError> {
    let artifact = file.artifact();
    Ok(Builder {
        policy: TargetPolicy::Dom,
        original_attributes: Some(OriginalAttributeCursor::new(file, artifact)?),
        node_count: artifact.node_count(),
        frames: Vec::new(),
        nodes: SideTable::with_capacity(artifact.node_count() as usize),
        controls: SideTable::new(),
        dom: Some(DomBuilder::new(
            &LiteralExpressions,
            artifact.root().ops.len(),
            Some(file),
            &NoReads,
        )),
        ssr: None,
        vapor: None,
    })
}

fn finish_client<'owner, 'arena>(
    mut client: ClientPass<'owner, 'arena>,
    view: NativeTemplateView<'owner, 'arena>,
    file: &'owner L2Ir<'arena>,
) -> Result<TemplateDomAnalysis<'owner, 'arena>, DecisionBuildError> {
    let original_attributes = client
        .original_attributes
        .take()
        .map(OriginalAttributeCursor::finish)
        .transpose()?;
    let dom = client.dom.take().map(DomBuilder::finish);
    let analysis = DecisionAnalysis {
        artifact: file.artifact(),
        tables: client.finish()?,
        original_attributes,
        dom,
        ssr: None,
        vapor: None,
    };
    Ok(TemplateDomAnalysis::from_pair(CompletedTarget {
        view,
        file,
        analysis,
    }))
}

// A refusal from either consumer stops only that consumer.
#[inline]
fn visit<T>(
    pass: &mut Result<T, DecisionBuildError>,
    observe: impl FnOnce(&mut T) -> Result<(), DecisionBuildError>,
) {
    if let Ok(state) = pass
        && let Err(error) = observe(state)
    {
        *pass = Err(error);
    }
}

/// Produce the original DOM analysis and the facts consumed by the SSR writer.
/// Each policy owns its attribute cursor and keeps its own refusal.
pub fn build_client_server_decisions<'owner, 'arena>(
    owner: &'owner NativeTemplateFile<'arena>,
) -> Result<ClientServerDecisions<'owner, 'arena>, NativeTemplateIssue> {
    let view = owner.view()?;
    if owner.selected().has_styles() {
        let span = Span::new(
            0,
            owner.selected().component().block().root_source().len() as u32,
        );
        return Ok(ClientServerDecisions {
            dom: build_template_dom_decisions(view),
            ssr: Err(SsrTemplateBuildError::StyledSource { span }),
        });
    }
    let Some(file) = view.file() else {
        return Ok(ClientServerDecisions::refused(
            DecisionBuildError::IncompleteFile,
        ));
    };
    if !file.native_text_values().is_empty() {
        return Ok(ClientServerDecisions::refused(
            DecisionBuildError::PreparedTextProfile,
        ));
    }
    let artifact = file.artifact();
    let mut server = RenderPass::new(file);
    // Small files finish the client's temporary frames before opening the
    // server's. Larger files amortize both passes in a single event traversal.
    let dom = if artifact.node_count() <= 64 {
        let dom = build_template_dom_decisions(view);
        if server.is_ok()
            && artifact
                .visit_events(&mut |event| visit(&mut server, |s| s.visit(event)))
                .is_err()
        {
            server = Err(DecisionBuildError::NodeLimit);
        }
        dom
    } else {
        let mut client = client_pass(file);
        if (client.is_ok() || server.is_ok())
            && artifact
                .visit_events(&mut |event| {
                    visit(&mut client, |c| c.visit(event));
                    visit(&mut server, |s| s.visit(event));
                })
                .is_err()
        {
            return Ok(ClientServerDecisions::refused(
                DecisionBuildError::NodeLimit,
            ));
        }
        client.and_then(|c| finish_client(c, view, file))
    };
    let ssr = server
        .and_then(|s| s.finish(owner, file))
        .map_err(SsrTemplateBuildError::Decision);
    Ok(ClientServerDecisions { dom, ssr })
}
