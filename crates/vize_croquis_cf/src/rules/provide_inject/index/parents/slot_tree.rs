use super::SlotScopes;
use crate::registry::FileId;
use vize_carton::FxHashMap;
use vize_croquis::provide::ProvideEntry;

impl SlotScopes {
    pub(in crate::rules::provide_inject::index) fn tree_edge_visible(
        &self,
        parent: FileId,
        child: FileId,
        ancestry: &[FileId],
        provides: &FxHashMap<FileId, Vec<ProvideEntry>>,
    ) -> bool {
        if self
            .bare_parents
            .get(&child)
            .is_some_and(|owners| owners.contains(&parent))
        {
            return true;
        }
        let Some(scopes) = self.scopes.get(&child) else {
            return true;
        };
        let mut relevant = false;
        for scope in scopes {
            if scope.owner == parent {
                relevant = true;
                if !scope
                    .receivers
                    .iter()
                    .any(|host| provides.contains_key(host))
                {
                    return true;
                }
                continue;
            }
            let Some(position) = scope.receivers.iter().position(|host| *host == parent) else {
                continue;
            };
            relevant = true;
            let mut actual = ancestry.iter().rev();
            let expected = scope
                .receivers
                .iter()
                .skip(position)
                .copied()
                .filter(|host| provides.contains_key(host))
                .chain(std::iter::once(scope.owner));
            if expected
                .into_iter()
                .all(|host| actual.next() == Some(&host))
            {
                return true;
            }
        }
        !relevant
    }
}
