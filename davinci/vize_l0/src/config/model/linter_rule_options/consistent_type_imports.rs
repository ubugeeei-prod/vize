//! Options for `script/consistent-type-imports`.
//!
//! `prefer`, `fixStyle` and `disallowTypeAnnotations` follow typescript-eslint
//! `consistent-type-imports`, so a migrated configuration keeps working.
//! `emitDecoratorMetadata` stands in for the compiler option typescript-eslint
//! reads from `tsconfig.json`: lint does not load the TypeScript project.

use serde::{Deserialize, Serialize};

/// Which import kind an import that is only used as a type takes.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TypeImportPreference {
    /// `import type` for every import that is only used as a type.
    #[default]
    TypeImports,
    /// Never `import type`.
    NoTypeImports,
}

/// How the fix writes an import that is only used as a type.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TypeImportFixStyle {
    /// `import type { A } from "m"`.
    #[default]
    SeparateTypeImports,
    /// `import { type A } from "m"`.
    InlineTypeImports,
}

/// `script/consistent-type-imports`, in the typescript-eslint shape.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct ConsistentTypeImportsOptions {
    /// `type-imports` (default) or `no-type-imports`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefer: Option<TypeImportPreference>,
    /// `separate-type-imports` (default) or `inline-type-imports`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fix_style: Option<TypeImportFixStyle>,
    /// Report `import("m")` type annotations. Defaults to `true`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disallow_type_annotations: Option<bool>,
    /// Whether the project compiles with `emitDecoratorMetadata`. Defaults to
    /// `true`: a type that a decorated signature serializes then counts as a
    /// value use, so the fix never changes emitted metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emit_decorator_metadata: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::{ConsistentTypeImportsOptions, TypeImportFixStyle, TypeImportPreference};

    #[test]
    fn the_typescript_eslint_shape_parses_and_unknown_keys_are_refused() {
        let options: ConsistentTypeImportsOptions = serde_json::from_value(serde_json::json!({
            "prefer": "no-type-imports",
            "fixStyle": "inline-type-imports",
            "disallowTypeAnnotations": false,
            "emitDecoratorMetadata": false,
        }))
        .unwrap();
        assert_eq!(
            options,
            ConsistentTypeImportsOptions {
                prefer: Some(TypeImportPreference::NoTypeImports),
                fix_style: Some(TypeImportFixStyle::InlineTypeImports),
                disallow_type_annotations: Some(false),
                emit_decorator_metadata: Some(false),
            }
        );
        assert_eq!(
            serde_json::from_value::<ConsistentTypeImportsOptions>(serde_json::json!({})).unwrap(),
            ConsistentTypeImportsOptions::default()
        );
        assert!(
            serde_json::from_value::<ConsistentTypeImportsOptions>(
                serde_json::json!({ "prefer": "type" })
            )
            .is_err()
        );
        assert!(
            serde_json::from_value::<ConsistentTypeImportsOptions>(
                serde_json::json!({ "fixstyle": "inline-type-imports" })
            )
            .is_err()
        );
    }
}
