//! `[Symbol.dispose]` members, read by no-misused-promises' variable checks.
use super::PromiseChecker;
use corsa::api::TypeResponse;

impl PromiseChecker<'_> {
    /// typescript-eslint's `hasWellKnownSymbolWithThenableReturn` (`thenable`)
    /// or `hasWellKnownSymbolWithVoidReturn`: a union part of the apparent
    /// type has a `[Symbol.dispose]` member whose type is a thenable- or
    /// void-returning function.
    pub(super) async fn dispose_returns(
        &mut self,
        ty: &TypeResponse,
        thenable: bool,
    ) -> corsa::Result<bool> {
        let apparent = self.apparent(ty).await?;
        for part in self.union_parts(&apparent).await? {
            // `getWellKnownSymbolPropertyOfType`: the checker names the
            // member `__@dispose@<id>`.
            let member = self
                .session
                .get_properties_of_type(part.id.clone())
                .await
                .unwrap_or_default()
                .into_iter()
                .find(|property| {
                    property
                        .name
                        .strip_prefix("__@dispose@")
                        .is_some_and(|id| !id.is_empty())
                });
            let Some(member) = member else {
                continue;
            };
            let Some(member) = self.symbol_type(&member.id).await? else {
                continue;
            };
            let found = if thenable {
                self.thenable_function(&member).await?
            } else {
                self.void_function(&member).await?
            };
            if found {
                return Ok(true);
            }
        }
        Ok(false)
    }
}
