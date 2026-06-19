//! TypeScript type resolution for Vue compiler macros.
//!
//! Provides type extraction and resolution for defineProps, defineEmits, etc.
//! Supports:
//! - Inline object types: `defineProps<{ msg: string }>()`
//! - Type references: `defineProps<Props>()`
//! - External imports (future): `import type { Props } from './types'`

use vize_carton::{CompactString, FxHashMap, String};

/// Resolved type information
#[derive(Debug, Clone)]
pub struct ResolvedType {
    /// Original type string
    pub raw: CompactString,
    /// Whether this is a reference to another type
    pub is_reference: bool,
    /// Resolved body (for object types)
    pub body: Option<CompactString>,
}

/// Extracted property from a type definition
#[derive(Debug, Clone)]
pub struct TypeProperty {
    /// Property name
    pub name: CompactString,
    /// Property type (as string)
    pub prop_type: Option<CompactString>,
    /// Whether the property is optional
    pub optional: bool,
}

/// Type definitions collected from script
#[derive(Debug, Default)]
pub struct TypeDefinitions {
    /// Interface definitions (name -> body)
    pub interfaces: FxHashMap<CompactString, CompactString>,
    /// Type alias definitions (name -> body)
    pub type_aliases: FxHashMap<CompactString, CompactString>,
    /// Imported types (name -> source path)
    pub imported_types: FxHashMap<CompactString, CompactString>,
}

impl TypeDefinitions {
    /// Create a new empty type definitions store
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an interface definition
    #[inline]
    pub fn add_interface(
        &mut self,
        name: impl Into<CompactString>,
        body: impl Into<CompactString>,
    ) {
        self.interfaces.insert(name.into(), body.into());
    }

    /// Add a type alias definition
    #[inline]
    pub fn add_type_alias(
        &mut self,
        name: impl Into<CompactString>,
        body: impl Into<CompactString>,
    ) {
        self.type_aliases.insert(name.into(), body.into());
    }

    /// Add an imported type
    #[inline]
    pub fn add_imported_type(
        &mut self,
        name: impl Into<CompactString>,
        source: impl Into<CompactString>,
    ) {
        self.imported_types.insert(name.into(), source.into());
    }

    /// Resolve a type reference
    pub fn resolve(&self, type_name: &str) -> Option<&CompactString> {
        self.interfaces
            .get(type_name)
            .or_else(|| self.type_aliases.get(type_name))
    }

    /// Check if a type is defined locally
    #[inline]
    pub fn is_defined(&self, type_name: &str) -> bool {
        self.interfaces.contains_key(type_name) || self.type_aliases.contains_key(type_name)
    }

    /// Check if a type is imported
    #[inline]
    pub fn is_imported(&self, type_name: &str) -> bool {
        self.imported_types.contains_key(type_name)
    }

    /// Merge another set of definitions in, keeping existing entries on a name
    /// clash. Used to fold a plain `<script>`'s local types into a
    /// `<script setup>` summary, which keeps precedence for setup-local data.
    pub fn merge_keep_existing(&mut self, other: TypeDefinitions) {
        for (name, body) in other.interfaces {
            self.interfaces.entry(name).or_insert(body);
        }
        for (name, body) in other.type_aliases {
            self.type_aliases.entry(name).or_insert(body);
        }
        for (name, source) in other.imported_types {
            self.imported_types.entry(name).or_insert(source);
        }
    }
}

/// Type resolver for Vue compiler macros
#[derive(Debug, Default)]
pub struct TypeResolver {
    /// Collected type definitions
    definitions: TypeDefinitions,
}

impl TypeResolver {
    /// Create a new type resolver
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Get type definitions
    #[inline]
    pub fn definitions(&self) -> &TypeDefinitions {
        &self.definitions
    }

    /// Get mutable type definitions
    #[inline]
    pub fn definitions_mut(&mut self) -> &mut TypeDefinitions {
        &mut self.definitions
    }

    /// Add an interface definition
    #[inline]
    pub fn add_interface(
        &mut self,
        name: impl Into<CompactString>,
        body: impl Into<CompactString>,
    ) {
        self.definitions.add_interface(name, body);
    }

    /// Add a type alias definition
    #[inline]
    pub fn add_type_alias(
        &mut self,
        name: impl Into<CompactString>,
        body: impl Into<CompactString>,
    ) {
        self.definitions.add_type_alias(name, body);
    }

    /// Fold another resolver's definitions in, keeping existing entries on a
    /// name clash (the receiver wins). Used when merging a plain `<script>`'s
    /// local types into a `<script setup>` summary.
    #[inline]
    pub fn merge_keep_existing(&mut self, other: TypeResolver) {
        self.definitions.merge_keep_existing(other.definitions);
    }

    /// Extract properties from type arguments
    ///
    /// Handles:
    /// - Inline object types: `{ msg: string, count?: number }`
    /// - Type references: `Props` (resolved via definitions)
    pub fn extract_properties(&self, type_args: &str) -> Vec<TypeProperty> {
        let content = type_args.trim();

        // Resolve type reference if not an inline object type
        let resolved_content = if content.starts_with('{') {
            // Inline object type - strip braces
            if content.ends_with('}') {
                &content[1..content.len() - 1]
            } else {
                content
            }
        } else {
            // Type reference - look up in definitions
            if let Some(body) = self.definitions.resolve(content) {
                let body = body.trim();
                if body.starts_with('{') && body.ends_with('}') {
                    &body[1..body.len() - 1]
                } else {
                    body
                }
            } else {
                // Unresolved type reference - return empty
                return Vec::new();
            }
        };

        self.parse_type_members(resolved_content)
    }

    /// Parse type members from a type body string
    fn parse_type_members(&self, content: &str) -> Vec<TypeProperty> {
        let mut properties = Vec::new();
        let mut depth = 0;
        let mut current = String::default();
        let mut prev = '\0';

        for c in content.chars() {
            match c {
                '{' | '<' | '(' | '[' => {
                    depth += 1;
                    current.push(c);
                }
                '}' | ')' | ']' => {
                    depth -= 1;
                    current.push(c);
                }
                '>' if prev != '=' => {
                    depth -= 1;
                    current.push(c);
                }
                ',' | ';' | '\n' if depth == 0 => {
                    if let Some(prop) = self.parse_single_property(&current) {
                        properties.push(prop);
                    }
                    current.clear();
                }
                _ => current.push(c),
            }
            prev = c;
        }

        // Process last segment
        if let Some(prop) = self.parse_single_property(&current) {
            properties.push(prop);
        }

        properties
    }

    /// Parse a single property from a type definition segment
    fn parse_single_property(&self, segment: &str) -> Option<TypeProperty> {
        let trimmed = segment.trim();
        if trimmed.is_empty() {
            return None;
        }

        // Parse "name?: Type" or "name: Type"
        let colon_pos = trimmed.find(':')?;
        let name_part = &trimmed[..colon_pos];
        let type_part = &trimmed[colon_pos + 1..];

        let optional = name_part.ends_with('?');
        let name = name_part.trim().trim_end_matches('?').trim();

        if name.is_empty() || !is_valid_identifier(name) {
            return None;
        }

        Some(TypeProperty {
            name: CompactString::new(name),
            prop_type: Some(CompactString::new(type_part.trim())),
            optional,
        })
    }

    /// Extract emit event names from emit type arguments
    ///
    /// Handles:
    /// - Call signatures: `{ (e: 'click'): void }`
    /// - Object type: `{ click: [] }` (Vue 3.3+)
    pub fn extract_emits(&self, type_args: &str) -> Vec<CompactString> {
        let content = type_args.trim();
        let mut emits = Vec::new();

        // Resolve if type reference
        let resolved = if content.starts_with('{') {
            if content.ends_with('}') {
                &content[1..content.len() - 1]
            } else {
                content
            }
        } else if let Some(body) = self.definitions.resolve(content) {
            let body = body.trim();
            if body.starts_with('{') && body.ends_with('}') {
                &body[1..body.len() - 1]
            } else {
                body
            }
        } else {
            return emits;
        };

        // Parse call signatures: (e: 'click'): void
        // or object properties: click: []
        // Split on semicolons only to avoid splitting call signature parameters
        for segment in resolved.split(&[';', '\n'][..]) {
            let trimmed = segment.trim();

            // Call signature: (e: 'eventName'): returnType
            if trimmed.starts_with('(') {
                if let Some(event_name) = extract_event_from_call_signature(trimmed) {
                    emits.push(event_name);
                }
            }
            // Object property: eventName: PayloadType
            // For object syntax, split on comma
            else if !trimmed.is_empty() {
                for prop in trimmed.split(',') {
                    let prop = prop.trim();
                    if let Some(colon_pos) = prop.find(':') {
                        let name = prop[..colon_pos].trim();
                        if !name.is_empty() && is_valid_identifier(name) {
                            emits.push(CompactString::new(name));
                        }
                    }
                }
            }
        }

        emits
    }
}

/// Extract event name from a call signature like `(e: 'click', payload: number): void`
fn extract_event_from_call_signature(signature: &str) -> Option<CompactString> {
    // Find the first string literal after the colon
    let colon_pos = signature.find(':')?;
    let after_colon = &signature[colon_pos + 1..];

    // Find quoted string
    let quote_char = if after_colon.contains('\'') {
        '\''
    } else if after_colon.contains('"') {
        '"'
    } else {
        return None;
    };

    let start = after_colon.find(quote_char)? + 1;
    let rest = &after_colon[start..];
    let end = rest.find(quote_char)?;

    Some(CompactString::new(&rest[..end]))
}

/// Check if a string is a valid JavaScript identifier
fn is_valid_identifier(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    let mut chars = s.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !first.is_ascii_alphabetic() && first != '_' && first != '$' {
        return false;
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$')
}

#[cfg(test)]
mod tests;
