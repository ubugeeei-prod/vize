//! Fail-closed execution reachability over the already-authored fact packet.

use super::super::SetupContextTracker;
use super::Range;
use crate::binding_occurrences::{BindingOccurrences, OccurrenceBlock};
use crate::scope::{ScopeChain, ScopeData};
use crate::{TemplateExpression, TemplateExpressionKind};

impl SetupContextTracker {
    /// Proven client regions; function exemptions require complete, source-owned
    /// declaration/reference identities, with every incoming use accounted for.
    #[doc(hidden)]
    pub fn client_only_browser_ranges(
        &self,
        scopes: &ScopeChain,
        template: &[TemplateExpression],
    ) -> Vec<Range> {
        let Some(facts) = self.ssr.as_ref() else {
            return Vec::new();
        };
        let mut regions = Vec::new();
        for &range in &facts.client_regions {
            append_execution_region(&mut regions, range, &facts.functions);
        }
        let Some(packet) = &facts.occurrences else {
            return regions;
        };
        for reference in packet.occurrences() {
            if reference.block == OccurrenceBlock::Script
                && self
                    .browser_globals
                    .iter()
                    .any(|(_, offset)| *offset == reference.start)
            {
                // A complete authored declaration owns this spelling, even
                // when it appears after the read. It is not a browser global.
                regions.push((reference.start, reference.end));
            }
        }
        let mut client_references = Vec::new();
        for &(callee, callback, watch) in &facts.vue_callbacks {
            if trusted_vue_callback(packet, scopes, callee, watch) {
                append_execution_region(&mut regions, callback, &facts.functions);
                client_references.push(callback);
            }
        }
        for &(range, probe) in &facts.global_client_regions {
            // The complete authored packet witnesses forward declarations too:
            // a locally-owned "window" never proves a browser-only branch.
            if !packet.occurrences().iter().any(|reference| {
                reference.block == OccurrenceBlock::Script
                    && (reference.start, reference.end) == probe
            }) {
                append_execution_region(&mut regions, range, &facts.functions);
            }
        }
        let functions = &facts.functions;
        if facts.opaque_functions {
            return regions;
        }
        let mut client = vec![false; functions.len()];
        let mut server = vec![false; functions.len()];
        let mut edges = Vec::new();
        for reference in packet.occurrences() {
            if reference.binding.block != OccurrenceBlock::Script {
                continue;
            }
            let declaration = (reference.binding.start, reference.binding.end);
            let Some(target) = functions
                .iter()
                .position(|(_, at)| *at == Some(declaration))
            else {
                continue;
            };
            let at = (reference.start, reference.end);
            if reference.block == OccurrenceBlock::Template {
                if template.iter().any(|expression| {
                    expression.kind == TemplateExpressionKind::VOn
                        && contains((expression.start, expression.end), at.0)
                        && at.1 <= expression.end
                }) {
                    mark(&mut client, target);
                } else {
                    mark(&mut server, target);
                }
                continue;
            }
            if reference.block != OccurrenceBlock::Script {
                mark(&mut server, target);
                continue;
            }
            if client_references.contains(&at) || regions.iter().any(|range| contains(*range, at.0))
            {
                mark(&mut client, target);
                continue;
            }
            if !facts.calls.contains(&at) {
                // Alias, mutation, return, export, or unknown callback escape.
                mark(&mut server, target);
                continue;
            }
            let owner = functions
                .iter()
                .enumerate()
                .filter(|(_, (range, _))| contains(*range, at.0))
                .min_by_key(|(_, (range, _))| range.1.saturating_sub(range.0));
            match owner {
                Some((source, (_, Some(_)))) => edges.push((source, target)),
                _ => mark(&mut server, target),
            }
        }
        // Both roots propagate: a function called by a handler and setup still
        // runs during SSR. Cycles cannot manufacture a client root.
        loop {
            let mut changed = false;
            for &(source, target) in &edges {
                for flags in [&mut client, &mut server] {
                    if flags.get(source).copied() == Some(true)
                        && let Some(target) = flags.get_mut(target)
                        && !*target
                    {
                        *target = true;
                        changed = true;
                    }
                }
            }
            if !changed {
                break;
            }
        }
        for (((range, declaration), client), server) in functions.iter().zip(client).zip(server) {
            if declaration.is_some() && client && !server {
                regions.push(*range);
            }
        }
        regions
    }
}

fn mark(flags: &mut [bool], index: usize) {
    if let Some(flag) = flags.get_mut(index) {
        *flag = true;
    }
}

fn contains((start, end): Range, offset: u32) -> bool {
    start <= offset && offset < end
}

fn trusted_vue_callback(
    packet: &BindingOccurrences,
    scopes: &ScopeChain,
    callee: Range,
    watch: bool,
) -> bool {
    let Some(reference) = packet.occurrences().iter().find(|reference| {
        reference.block == OccurrenceBlock::Script && (reference.start, reference.end) == callee
    }) else {
        return false;
    };
    let Some(binding) = packet
        .bindings()
        .find(|binding| binding.identity == reference.binding)
    else {
        return false;
    };
    scopes.iter().any(|scope| {
        let ScopeData::ExternalModule(import) = scope.data() else {
            return false;
        };
        import.source == "vue"
            && !import.is_type_only
            && scope.span.start <= binding.identity.start
            && binding.identity.end <= scope.span.end
            && import.exports.iter().any(|export| {
                export.local_name == binding.name
                    && if watch {
                        export.export_name == "watch"
                    } else {
                        matches!(
                            export.export_name.as_str(),
                            "onMounted"
                                | "onBeforeMount"
                                | "onUnmounted"
                                | "onBeforeUnmount"
                                | "onUpdated"
                                | "onBeforeUpdate"
                                | "onActivated"
                                | "onDeactivated"
                        )
                    }
            })
    })
}

fn append_execution_region(
    regions: &mut Vec<Range>,
    range: Range,
    functions: &[(Range, Option<Range>)],
) {
    // A hoisted function's textual position after an SSR-return guard does not
    // prove its execution context. Its callers must establish that separately.
    let mut parts = vec![range];
    for &(function, _) in functions {
        if function.0 <= range.0 || function.0 >= range.1 {
            continue;
        }
        let mut remaining = Vec::new();
        for (start, end) in parts {
            if function.1 <= start || function.0 >= end {
                remaining.push((start, end));
            } else {
                if start < function.0 {
                    remaining.push((start, function.0));
                }
                if function.1 < end {
                    remaining.push((function.1, end));
                }
            }
        }
        parts = remaining;
    }
    regions.extend(parts);
}
