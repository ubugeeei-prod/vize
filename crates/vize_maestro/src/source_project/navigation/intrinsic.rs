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
        self.render_intrinsic_tag(uri, position, |target| async { Some(target) })
            .await
    }

    /// Keep the original document lease through asynchronous documentation work.
    /// An owned target alone cannot authorize publication after an edit or close.
    pub(crate) async fn render_intrinsic_tag<T, F, Fut>(
        &self,
        uri: &Url,
        position: Position,
        render: F,
    ) -> Result<Option<T>, NavigationRefusal>
    where
        F: FnOnce(IntrinsicTagTarget) -> Fut,
        Fut: std::future::Future<Output = Option<T>>,
    {
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
                let target = worker.intrinsic_tag(position).await?;
                let rendered = match target {
                    Some(target) => render(target).await,
                    None => None,
                };
                Ok((worker.profile(), Ok(rendered)))
            })
            .await
            .map_err(NavigationRefusal::Host)?;
        ready
            .publish(|response| self.checked_response(response))
            .map_err(NavigationRefusal::Host)?
    }
}
