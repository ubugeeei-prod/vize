//! Cross-component composition (Davinci P4-11b): the nesting no per-file
//! check can see.
//!
//! A template is checked with its mount point unknown, so a component whose
//! root is a `<div>` is fine on its own — and wrong the moment a parent
//! renders it inside a `<p>`. Composition re-checks each statically resolved
//! child template in the chain its usage site is rendered in, and reports
//! every verdict that the parent's context proves and the child's own check
//! could not. Soundness of `unknown` (every per-file verdict holds in every
//! faithful context) is what makes this sound: composition only ever turns
//! unknowns into proofs.
//!
//! Declared domain: components resolved through the module graph (no
//! name-based fallback, no dynamic `:is`); a `<slot>` pass-through is
//! unknown — whether fallback content renders depends on what the usage
//! passes — and slot content keeps its per-file verdicts. A hyphenated tag
//! is a usage too (`<my-card />` renders `MyCard`); the lint parser leaves
//! it as an element, which would hide a `<div>` root inside a `<p>`.
//!
//! A child subtree guarded by `v-if="prop"` is left out when the usage does
//! not pass `prop` and the caller's oracle proves its absent value falsy
//! (FP-3): that subtree is not rendered at this usage.

use vize_s0::FxHashSet;

use super::chain::Chain;
use super::check::{Context, Report, Verdict, check, check_pruned, check_with};
use super::class::ViolationClass;
use super::skeleton::{Skeleton, component_usage_name};

/// How deep component chains are followed.
const MAX_DEPTH: usize = 16;

/// A violation proven only in a parent's context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComposedFinding {
    /// The component usages from the checked file down to the template that
    /// holds the violating node: `(file, component node)`, outermost first.
    pub usages: Vec<(u32, u32)>,
    /// The file holding the violating node.
    pub file: u32,
    /// The violating node.
    pub node: u32,
    /// The violation class.
    pub class: ViolationClass,
    /// The deciding ancestor: `(file, node)`, possibly in a parent file.
    pub evidence: Option<(u32, u32)>,
}

/// Compose every file with the children it renders. `skeletons[i]` is file
/// `i`; `resolve(file, component)` names the file a component usage renders,
/// or `None` when it is not statically resolved.
pub fn compose(
    skeletons: &[Skeleton],
    resolve: &dyn Fn(u32, &str) -> Option<u32>,
) -> Vec<ComposedFinding> {
    compose_with(skeletons, resolve, &|_, _| false)
}

/// [`compose`] pruning guarded subtrees: `absent_falsy(file, prop)` holds
/// when an unpassed `prop` of `file` is proven falsy (no default, or a falsy
/// literal one) and the template identifier `prop` names that prop.
pub fn compose_with(
    skeletons: &[Skeleton],
    resolve: &dyn Fn(u32, &str) -> Option<u32>,
    absent_falsy: &dyn Fn(u32, &str) -> bool,
) -> Vec<ComposedFinding> {
    let standalone: Vec<Report> = skeletons
        .iter()
        .enumerate()
        .map(|(id, skeleton)| check(skeleton, id as u32, &Context::Truncated))
        .collect();
    let mut composer = Composer {
        skeletons,
        standalone: &standalone,
        resolve,
        absent_falsy,
        findings: Vec::new(),
        seen: FxHashSet::default(),
    };
    for (id, skeleton) in skeletons.iter().enumerate() {
        let mut usages = Vec::new();
        check_with(
            skeleton,
            id as u32,
            &Context::Truncated,
            false,
            &mut |node, chain| {
                usages.push((node, chain.clone()));
            },
        );
        for (node, chain) in usages {
            composer.usage(&[(id as u32, node)], chain);
        }
    }
    composer.findings
}

struct Composer<'a> {
    skeletons: &'a [Skeleton],
    standalone: &'a [Report],
    resolve: &'a dyn Fn(u32, &str) -> Option<u32>,
    absent_falsy: &'a dyn Fn(u32, &str) -> bool,
    findings: Vec<ComposedFinding>,
    seen: FxHashSet<(Vec<(u32, u32)>, u32)>,
}

impl Composer<'_> {
    /// Check the template `path`'s last usage renders, in `chain`.
    fn usage(&mut self, path: &[(u32, u32)], chain: Chain) {
        let Some(&(file, node)) = path.last() else {
            return;
        };
        let Some(name) = component_usage_name(self.skeletons[file as usize].node(node)) else {
            return;
        };
        let Some(child) = (self.resolve)(file, name) else {
            return;
        };
        // A usage chain that revisits a file is recursion: stop there.
        if path.len() > MAX_DEPTH || path.iter().any(|(visited, _)| *visited == child) {
            return;
        }
        let skeleton = &self.skeletons[child as usize];
        let parent = &self.skeletons[file as usize];
        let pruned: Vec<u32> = (skeleton.props.guards.iter())
            .filter(|(_, prop)| {
                parent.passes(node, prop) == Some(false) && (self.absent_falsy)(child, prop)
            })
            .map(|(guard, _)| *guard)
            .collect();
        let mut nested = Vec::new();
        let report = check_pruned(
            skeleton,
            child,
            &Context::Chain(chain),
            true,
            &pruned,
            &mut |node, chain| {
                nested.push((node, chain.clone()));
            },
        );
        let standalone = &self.standalone[child as usize];
        for (index, verdict) in report.verdicts.iter().enumerate() {
            let Verdict::Proven { class, evidence } = *verdict else {
                continue;
            };
            if standalone.verdicts[index] == *verdict {
                continue;
            }
            let key = (path.to_vec(), index as u32);
            if self.seen.insert(key) {
                self.findings.push(ComposedFinding {
                    usages: path.to_vec(),
                    file: child,
                    node: index as u32,
                    class,
                    evidence,
                });
            }
        }
        for (node, chain) in nested {
            // A usage whose chain no longer reaches into the parent's context
            // (behind a boundary) composes exactly as in the child's own pass.
            let inherits = chain
                .frames
                .iter()
                .any(|frame| frame.origin.is_some_and(|(owner, _)| owner != child));
            if !inherits {
                continue;
            }
            let mut deeper = path.to_vec();
            deeper.push((child, node));
            self.usage(&deeper, chain);
        }
    }
}
