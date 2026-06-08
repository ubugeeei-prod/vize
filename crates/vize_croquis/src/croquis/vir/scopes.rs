use std::fmt::Write;

use vize_carton::{FxHashMap, String};

use super::super::Croquis;

impl Croquis {
    pub(super) fn write_scopes(&self, output: &mut String) {
        if self.scopes.is_empty() {
            return;
        }

        // Build a map from scope ID -> prefixed display ID
        // Separate counters for ~, !, # prefixes
        let mut prefix_counters: FxHashMap<&str, u32> = FxHashMap::default();
        let mut id_to_display: FxHashMap<u32, String> = FxHashMap::default();

        // Helper to determine effective prefix by checking parent chain
        // If any ancestor is ClientOnly, child scopes should also be !
        // If any ancestor is server-only, child scopes should also be #
        let get_effective_prefix = |scope: &crate::scope::Scope| -> &'static str {
            // First check the scope's own prefix
            let own_prefix = scope.kind.prefix();
            if own_prefix != "~" {
                return own_prefix;
            }

            // Check parent chain for client-only or server-only context
            let mut visited: vize_carton::SmallVec<[crate::scope::ScopeId; 8]> =
                vize_carton::SmallVec::new();
            let mut queue: vize_carton::SmallVec<[crate::scope::ScopeId; 8]> =
                scope.parents.iter().copied().collect();

            while let Some(parent_id) = queue.pop() {
                if visited.contains(&parent_id) {
                    continue;
                }
                visited.push(parent_id);

                if let Some(parent) = self.scopes.get_scope(parent_id) {
                    let parent_prefix = parent.kind.prefix();
                    if parent_prefix == "!" {
                        return "!"; // Client-only context propagates down
                    }
                    if parent_prefix == "#" {
                        return "#"; // Server-only context propagates down
                    }
                    // Add grandparents to queue
                    for &gp in &parent.parents {
                        if !visited.contains(&gp) {
                            queue.push(gp);
                        }
                    }
                }
            }

            "~" // Default to universal
        };

        for scope in self.scopes.iter() {
            let prefix = get_effective_prefix(scope);
            let counter = prefix_counters.entry(prefix).or_insert(0);
            #[allow(clippy::disallowed_macros)]
            let display_id = format!("{}{}", prefix, *counter);
            id_to_display.insert(scope.id.as_u32(), display_id.into());
            *counter += 1;
        }

        writeln!(output, "[scopes]").ok();
        for scope in self.scopes.iter() {
            let bd_count = scope.bindings().count();

            // Get scope display ID with prefix
            let scope_id_display = id_to_display
                .get(&scope.id.as_u32())
                .map(|s| s.as_str())
                .unwrap_or("?");

            // Build parent references from the parents list using display IDs
            let par = if scope.parents.is_empty() {
                String::default()
            } else {
                let refs: Vec<_> = scope
                    .parents
                    .iter()
                    .filter_map(|p| id_to_display.get(&p.as_u32()))
                    .map(|s| s.as_str())
                    .collect();
                if refs.is_empty() {
                    String::default()
                } else {
                    {
                        #[allow(clippy::disallowed_macros)]
                        let s = format!(" < {}", refs.join(", "));
                        s.into()
                    }
                }
            };

            if bd_count > 0 {
                let bd: Vec<_> = scope.bindings().map(|(n, _)| n).collect();
                writeln!(
                    output,
                    "{} {} @{}:{} [{}]{}",
                    scope_id_display,
                    scope.display_name(),
                    scope.span.start,
                    scope.span.end,
                    bd.join(","),
                    par
                )
                .ok();
            } else {
                writeln!(
                    output,
                    "{} {} @{}:{}{}",
                    scope_id_display,
                    scope.display_name(),
                    scope.span.start,
                    scope.span.end,
                    par
                )
                .ok();
            }
        }
        writeln!(output).ok();
    }
}
