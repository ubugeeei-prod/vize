//! One projection-owned selection for generated component declarations and uses.

use std::cell::RefCell;

use vize_carton::{
    CompactString, FxHashMap, FxHashSet, String, append, camelize, capitalize, cstr,
};
use vize_croquis::Croquis;

use super::component_reference::{contains_compact_name, has_type_only_component_candidate};
use super::generator::extract_declared_name;
use super::helpers::to_safe_identifier;
use super::types::VirtualTsOptions;

#[derive(Default)]
struct Aliases {
    references: FxHashMap<String, String>,
    inline_values: FxHashMap<String, String>,
    auto_imports: FxHashMap<String, String>,
}

/// Escape underscores too: authored encoded-looking text cannot claim another tag.
fn fragment(name: &str) -> String {
    let mut output = String::default();
    for character in name.chars() {
        if character.is_ascii_alphanumeric() || character == '$' {
            output.push(character);
        } else {
            append!(output, "_x{:x}_", character as u32);
        }
    }
    output
}

fn candidates(name: &str) -> [String; 3] {
    let camel = camelize(name);
    [capitalize(camel.as_str()), camel, String::from(name)]
}

impl Aliases {
    fn new(
        summary: &Croquis,
        options: &VirtualTsOptions,
        type_only: &FxHashSet<CompactString>,
        imported: &FxHashSet<&str>,
    ) -> Self {
        let names = vize_croquis::facts::used_component_name_list(summary);
        let inline_names: FxHashSet<&str> = options
            .auto_import_stubs
            .iter()
            .filter_map(|stub| extract_declared_name(stub))
            .collect();
        let mut identities = FxHashSet::<String>::default();
        for name in &names {
            let name = name.as_str();
            if has_type_only_component_candidate(type_only, name)
                || to_safe_identifier(name).as_str() != name
            {
                identities.insert(String::from(name));
            }
            for candidate in candidates(name) {
                if inline_names.contains(candidate.as_str())
                    && contains_compact_name(type_only, candidate.as_str())
                {
                    identities.insert(candidate);
                }
            }
        }
        if identities.is_empty() && inline_names.is_empty() {
            return Self::default();
        }
        let mut reserved: FxHashSet<String> =
            imported.iter().map(|name| String::from(*name)).collect();
        super::script_facts::with_bindings(summary, |bindings| {
            reserved.extend(bindings.typed().map(|(name, _)| String::from(name)));
            reserved.extend(bindings.spans().map(|(name, _)| String::from(name)));
        });
        for scope in summary.scopes.iter() {
            reserved.extend(scope.bindings().map(|(name, _)| String::from(name)));
        }
        reserved.extend(type_only.iter().map(|name| String::from(name.as_str())));
        reserved.extend(
            options
                .template_globals
                .iter()
                .map(|global| global.name.clone()),
        );
        reserved.extend(options.external_template_bindings.iter().cloned());
        reserved.extend(options.auto_import_binding_names());
        // Legal references retain their existing value/dynamic ownership. A
        // generated identifier cannot capture one of those authored tag names.
        reserved.extend(
            names
                .iter()
                .filter(|name| to_safe_identifier(name.as_str()).as_str() == name.as_str())
                .map(|name| String::from(name.as_str())),
        );
        let mut identities: Vec<_> = identities.into_iter().collect();
        identities.sort_unstable_by(|left, right| left.as_str().cmp(right.as_str()));
        let mut generated = FxHashMap::<String, String>::default();
        for identity in identities {
            let mut reference = cstr!("$Component_{}", fragment(identity.as_str()));
            // Encoded fragments end in at most one underscore, so a pair is
            // outside the encoded-name language and cannot merge two owners.
            while reserved.contains(reference.as_str()) {
                reference.push_str("__");
            }
            reserved.insert(reference.clone());
            generated.insert(identity, reference);
        }
        let mut output = Self::default();
        for name in &names {
            let name = name.as_str();
            if let Some(owner) = candidates(name)
                .into_iter()
                .find(|candidate| inline_names.contains(candidate.as_str()))
            {
                let reference = generated
                    .get(owner.as_str())
                    .filter(|_| contains_compact_name(type_only, owner.as_str()))
                    .cloned()
                    .unwrap_or_else(|| owner.clone());
                output
                    .inline_values
                    .insert(String::from(name), reference.clone());
                output.references.insert(String::from(name), reference);
            } else if let Some(reference) = generated.get(name) {
                output
                    .references
                    .insert(String::from(name), reference.clone());
            }
        }
        for owner in inline_names {
            if contains_compact_name(type_only, owner)
                && let Some(reference) = generated.get(owner)
            {
                output
                    .auto_imports
                    .insert(String::from(owner), reference.clone());
            }
        }
        output
    }
}

struct CachedAliases {
    summary: *const Croquis,
    options: *const VirtualTsOptions,
    type_only: *const FxHashSet<CompactString>,
    aliases: Aliases,
}

thread_local! {
    static CACHED: RefCell<Option<CachedAliases>> = const { RefCell::new(None) };
}

pub(super) struct AliasScope(Option<CachedAliases>);

impl Drop for AliasScope {
    fn drop(&mut self) {
        CACHED.with_borrow_mut(|cached| *cached = self.0.take());
    }
}

/// Share the deterministic plan across all declaration/reference emitters and
/// restore an outer projection on return or unwinding, like the bindings table.
pub(super) fn cache_aliases(
    summary: &Croquis,
    options: &VirtualTsOptions,
    type_only: &FxHashSet<CompactString>,
    imported: &FxHashSet<&str>,
) -> AliasScope {
    let current = CachedAliases {
        summary: std::ptr::from_ref(summary),
        options: std::ptr::from_ref(options),
        type_only: std::ptr::from_ref(type_only),
        aliases: Aliases::new(summary, options, type_only, imported),
    };
    AliasScope(CACHED.with_borrow_mut(|cached| cached.replace(current)))
}

fn with_aliases<T>(
    summary: &Croquis,
    options: &VirtualTsOptions,
    type_only: &FxHashSet<CompactString>,
    body: impl FnOnce(&Aliases) -> T,
) -> T {
    CACHED.with_borrow(|cached| {
        if let Some(cached) = cached.as_ref().filter(|cached| {
            std::ptr::eq(cached.summary, summary)
                && std::ptr::eq(cached.options, options)
                && std::ptr::eq(cached.type_only, type_only)
        }) {
            return body(&cached.aliases);
        }
        body(&Aliases::new(
            summary,
            options,
            type_only,
            &FxHashSet::default(),
        ))
    })
}

pub(super) fn reference(
    summary: &Croquis,
    options: &VirtualTsOptions,
    type_only: &FxHashSet<CompactString>,
    name: &str,
) -> String {
    with_aliases(summary, options, type_only, |aliases| {
        aliases
            .references
            .get(name)
            .cloned()
            .unwrap_or_else(|| to_safe_identifier(name))
    })
}

pub(super) fn inline_value(
    summary: &Croquis,
    options: &VirtualTsOptions,
    type_only: &FxHashSet<CompactString>,
    name: &str,
) -> Option<String> {
    with_aliases(summary, options, type_only, |aliases| {
        aliases.inline_values.get(name).cloned()
    })
}

pub(super) fn auto_import_alias(
    summary: &Croquis,
    options: &VirtualTsOptions,
    type_only: &FxHashSet<CompactString>,
    name: &str,
) -> Option<String> {
    with_aliases(summary, options, type_only, |aliases| {
        aliases.auto_imports.get(name).cloned()
    })
}

#[cfg(test)]
mod tests;
