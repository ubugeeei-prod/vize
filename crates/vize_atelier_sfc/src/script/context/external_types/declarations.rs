//! The declarations a walk reached, by name, and the merge of only those a
//! component can look up.
//!
//! A component merges every module of its walks into two flat maps, and the
//! first declaration of a name wins. Whether a declaration is merged depends
//! only on the declarations of its own name: on what the component already
//! declares under it and on which walked declaration of it comes first. So
//! the merge can be split by name, and a name the component never looks up
//! can be left out without changing any lookup.
//!
//! A component looks names up only while it resolves type text, and every
//! such text is its own source or the body of a declaration it merged.
//! Every name it looks up is therefore an identifier written in one of those
//! texts. Merging, from the component's source outwards, each name that the
//! texts merged so far spell, gives every lookup the declaration that merging
//! everything would have given it. Behind a barrel of many modules, a
//! component then copies the few declarations its props reach, not all of
//! them.

#![expect(
    clippy::disallowed_types,
    reason = "the components of a batch share one walk's declarations"
)]

use std::sync::Arc;

use oxc_syntax::identifier::is_identifier_part;
use vize_carton::{FxHashMap, FxHashSet, String};

use super::super::ScriptCompileContext;
use super::modules::{ModuleGraph, ModuleId};
use super::work::{Step, count};

/// The first declaration of each name in the modules of one walk, in the
/// order a component merges them, and the imports of those modules.
#[derive(Debug, Default)]
pub(crate) struct Declarations {
    names: FxHashMap<String, Declaration>,
    /// The imports of the walked modules, in merge order.
    pub(super) imports: Vec<(String, String)>,
    /// Whether a body spells an identifier with an escape, which the scan of
    /// identifiers cannot read. A component then merges every declaration.
    escaped: bool,
}

#[derive(Debug)]
struct Declaration {
    interface: bool,
    body: String,
    /// The identifiers written in `body`, each once.
    references: Box<[String]>,
}

impl Declarations {
    pub(super) fn of(graph: &ModuleGraph, modules: &[ModuleId]) -> Self {
        let mut declarations = Self::default();
        for summary in modules.iter().filter_map(|id| graph.summary(*id)) {
            // A module's interfaces merge before its type aliases.
            let interfaces = summary.interfaces.iter().map(|entry| (true, entry));
            let aliases = summary.type_aliases.iter().map(|entry| (false, entry));
            for (interface, (name, body)) in interfaces.chain(aliases) {
                if declarations.names.contains_key(name) {
                    continue;
                }
                declarations.escaped |= has_escape(body);
                let references = identifiers(body).map(String::from).collect();
                declarations.names.insert(
                    name.clone(),
                    Declaration {
                        interface,
                        body: body.clone(),
                        references,
                    },
                );
            }
            declarations.imports.extend(summary.imports.iter().cloned());
        }
        declarations
    }
}

impl ScriptCompileContext {
    /// Merge every declaration of `walks`, in order.
    pub(super) fn merge_declarations(&mut self, walks: &[Arc<Declarations>]) {
        for declarations in walks {
            for (name, declaration) in &declarations.names {
                self.merge_declaration(name, declaration);
            }
        }
    }

    /// Merge, of the declarations of `walks`, the ones a component whose
    /// source is `texts` can look up, as merging them all in order would.
    pub(crate) fn merge_referenced_declarations(
        &mut self,
        walks: &[Arc<Declarations>],
        texts: &[&str],
    ) {
        if walks.is_empty() {
            return;
        }
        // Both merges give every lookup the same declaration, so the cheaper
        // one runs. Scanning the texts costs about one declaration's merge per
        // `SCAN_BYTES_PER_DECLARATION` bytes: a walk of a few modules is merged
        // whole, a wide barrel by name.
        let declared: usize = walks
            .iter()
            .map(|declarations| declarations.names.len())
            .sum();
        let scanned: usize = texts.iter().map(|text| text.len()).sum();
        if declared.saturating_mul(SCAN_BYTES_PER_DECLARATION) <= scanned
            || walks.iter().any(|declarations| declarations.escaped)
            || texts.iter().any(|text| has_escape(text))
        {
            self.merge_declarations(walks);
            return;
        }
        self.merge_by_reference(walks, texts);
    }

    /// Merge the declarations of `walks` that `texts` name, then the ones
    /// the merged bodies name, until no merged body names another.
    pub(super) fn merge_by_reference(&mut self, walks: &[Arc<Declarations>], texts: &[&str]) {
        let mut seen = FxHashSet::default();
        let mut wanted: Vec<&str> = Vec::new();
        for text in texts {
            wanted.extend(identifiers(text).filter(|name| seen.insert(*name)));
        }
        while let Some(name) = wanted.pop() {
            // The first walk that declares a name decides it, as in the
            // merge of everything: a later declaration finds it taken.
            let Some(declaration) = walks
                .iter()
                .find_map(|declarations| declarations.names.get(name))
            else {
                continue;
            };
            if self.merge_declaration(name, declaration) {
                wanted.extend(
                    declaration
                        .references
                        .iter()
                        .map(String::as_str)
                        .filter(|name| seen.insert(*name)),
                );
            }
        }
    }

    /// Merge one imported declaration unless the name is already taken, in
    /// either map. A name declared in this SFC is that binding: an imported
    /// `interface Props` must not shadow a local `type Props`.
    fn merge_declaration(&mut self, name: &str, declaration: &Declaration) -> bool {
        if self.interfaces.contains_key(name) || self.type_aliases.contains_key(name) {
            return false;
        }
        count(Step::Declaration, 1);
        let map = if declaration.interface {
            &mut self.interfaces
        } else {
            &mut self.type_aliases
        };
        map.insert(name.into(), declaration.body.clone());
        true
    }
}

/// The bytes of component source one scan for identifiers reads in the time
/// one declaration takes to merge: two lookups, an insert and a copy of its
/// body.
const SCAN_BYTES_PER_DECLARATION: usize = 16;

/// An identifier escape (`A`, `\u{41}`) names an identifier that its
/// spelling does not show.
fn has_escape(text: &str) -> bool {
    text.contains("\\u")
}

/// Every maximal run of identifier characters in `text`. A run may start with
/// a digit or sit in a string or a comment: the names are a superset of the
/// identifiers written, which is all a merge needs.
fn identifiers(text: &str) -> impl Iterator<Item = &str> {
    text.split(|char: char| !is_identifier_part(char))
        .filter(|run| !run.is_empty())
}

#[cfg(test)]
mod tests {
    use super::identifiers;

    #[test]
    fn identifiers_are_the_runs_of_identifier_characters() {
        let found: Vec<_> = identifiers("{ a: Foo<Bar>; 'b-c': $d_1 | Ünï } // é0").collect();
        assert_eq!(found, ["a", "Foo", "Bar", "b", "c", "$d_1", "Ünï", "é0"]);
    }
}
