//! Intrinsic source names from the original retained JSX Program/File walk.

use super::{NativeNavigationProject, NavigationRefusal, Profile};
use tower_lsp::lsp_types::{Position, Range, Url};
use vize_l0::String;

/// An owned response, not a transferable syntax or File admission capability.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntrinsicTagTarget {
    pub name: String,
    pub range: Range,
}

impl NativeNavigationProject<'_> {
    pub async fn intrinsic_tag(
        &self,
        uri: &Url,
        position: Position,
    ) -> Result<Option<IntrinsicTagTarget>, NavigationRefusal> {
        let (query, _) = self
            .source
            .begin_query(uri)
            .map_err(NavigationRefusal::Host)?;
        let ready = query
            .run(|snapshot| async move {
                if !matches!(self.profile(&snapshot)?, Profile::Program(options) if options.jsx) {
                    return Err(NavigationRefusal::Language);
                }
                let worker = self.worker(snapshot)?;
                Ok((worker.profile(), worker.intrinsic_tag(position).await))
            })
            .await
            .map_err(NavigationRefusal::Host)?;
        ready
            .publish(|response| self.checked_response(response))
            .map_err(NavigationRefusal::Host)?
    }
}
