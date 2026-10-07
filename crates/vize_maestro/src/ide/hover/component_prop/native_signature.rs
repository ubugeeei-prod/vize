//! Keep authored prop types when the generic navigation anchor is `unknown`.

use tower_lsp::lsp_types::{Hover, HoverContents, MarkupKind};

use super::super::HoverBuilder;

pub(super) fn documented_prop_signature(
    builder: HoverBuilder,
    signature: &str,
    native: Option<&Hover>,
) -> HoverBuilder {
    let Some(Hover {
        contents: HoverContents::Markup(content),
        ..
    }) = native
    else {
        return builder.code("typescript", signature);
    };
    if content.kind != MarkupKind::Markdown {
        return builder.code("typescript", signature);
    }

    // Generic children deliberately use an open prop navigation anchor. Its
    // quick info can be `unknown` even though the current component interface
    // retains `T[]` or `string`. Preserve concrete native instantiations, and
    // replace only that exact unknown signature while keeping native JSDoc.
    if content.value.trim() == "unknown" {
        return builder.code("typescript", signature);
    }
    if let Some((native_signature, documentation)) = content
        .value
        .strip_prefix("```typescript\n")
        .and_then(|value| value.split_once("\n```"))
        && is_unknown_signature(native_signature)
    {
        let builder = builder.code("typescript", signature);
        let documentation = documentation.trim();
        return if documentation.is_empty() {
            builder
        } else {
            builder.description(documentation)
        };
    }
    builder.description(&content.value)
}

fn is_unknown_signature(signature: &str) -> bool {
    let signature = signature.trim();
    signature == "unknown"
        || signature
            .strip_prefix("(property) ")
            .and_then(|property| property.split_once(':'))
            .is_some_and(|(_, ty)| ty.trim() == "unknown")
}

#[cfg(test)]
#[path = "native_signature_tests.rs"]
mod tests;
