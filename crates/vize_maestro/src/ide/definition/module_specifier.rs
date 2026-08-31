//! Go-to-definition for import and export module specifiers.

use std::path::{Path, PathBuf};

use tower_lsp::lsp_types::{GotoDefinitionResponse, Location, Position, Range, Url};
use vize_canon::{PackageRouteResolver, PackageSourceOptions};
use vize_s0::cstr;

#[cfg(feature = "native")]
use vize_canon::CorsaBridge;

use super::IdeContext;
#[cfg(feature = "native")]
use crate::ide::corsa_support;

#[cfg(test)]
#[path = "module_specifier_tests.rs"]
mod tests;

pub(super) fn definition(ctx: &IdeContext<'_>) -> Option<GotoDefinitionResponse> {
    let specifier = specifier_at_offset(&ctx.content, ctx.offset)?;
    let target = resolve_specifier_with(
        ctx.uri,
        specifier,
        &mut ctx.state.package_route_resolver.lock(),
    )?;
    let uri = Url::from_file_path(target).ok()?;
    let origin = Position::new(0, 0);

    Some(GotoDefinitionResponse::Scalar(Location {
        uri,
        range: Range::new(origin, origin),
    }))
}

/// Ask TypeScript which conditional package target won, then map the exact
/// materialized identity recorded by Canon back to authored source. This keeps
/// manifest order, mode, custom conditions, and virtual-path spelling out of
/// Maestro.
#[cfg(feature = "native")]
pub(super) async fn definition_with_corsa(
    ctx: &IdeContext<'_>,
    bridge: &CorsaBridge,
) -> Option<GotoDefinitionResponse> {
    specifier_at_offset(&ctx.content, ctx.offset)?;
    let document = corsa_support::open_canonical_virtual_document(ctx, bridge).await?;
    let (line, character) =
        corsa_support::canonical_source_offset_to_position(&document, ctx.offset)?;
    let selected = bridge
        .definition(&document.request_uri, line, character)
        .await
        .ok()?;
    let mut mapped = selected
        .iter()
        .filter_map(|location| {
            corsa_support::map_canonical_corsa_location(ctx, &document, location).or_else(|| {
                corsa_support::map_canonical_materialized_module_location(&document, location)
            })
        })
        .map(pin_to_module_origin)
        .collect::<Vec<_>>();
    mapped.sort_by(|left, right| {
        left.uri
            .as_str()
            .cmp(right.uri.as_str())
            .then_with(|| left.range.start.cmp(&right.range.start))
    });
    mapped.dedup();
    match mapped.len() {
        0 => None,
        1 => Some(GotoDefinitionResponse::Scalar(mapped.pop()?)),
        _ => Some(GotoDefinitionResponse::Array(mapped)),
    }
}

/// A module specifier resolves to a *module*, not to a symbol inside it, so the
/// answer is the file identity pinned to its origin. Native TypeScript reports
/// the whole source file span for such a resolution, and forwarding that span
/// would make the editor select the entire target file (#3893).
#[cfg(feature = "native")]
fn pin_to_module_origin(mut location: Location) -> Location {
    let origin = Position::new(0, 0);
    location.range = Range::new(origin, origin);
    location
}

pub(super) fn specifier_at_offset(content: &str, offset: usize) -> Option<&str> {
    if offset > content.len() || !content.is_char_boundary(offset) {
        return None;
    }

    let line_start = content[..offset].rfind('\n').map_or(0, |index| index + 1);
    let line_end = content[offset..]
        .find('\n')
        .map_or(content.len(), |index| offset + index);
    let line = &content[line_start..line_end];
    let relative_offset = offset - line_start;
    let bytes = line.as_bytes();
    let mut cursor = 0;

    while cursor < bytes.len() {
        let quote = bytes[cursor];
        if quote != b'\'' && quote != b'"' {
            cursor += 1;
            continue;
        }

        let start = cursor;
        cursor += 1;
        while cursor < bytes.len() {
            if bytes[cursor] == b'\\' {
                cursor = (cursor + 2).min(bytes.len());
                continue;
            }
            if bytes[cursor] == quote {
                let end = cursor;
                if relative_offset > start
                    && relative_offset <= end
                    && is_module_context(&line[..start])
                {
                    return Some(&line[start + 1..end]);
                }
                cursor += 1;
                break;
            }
            cursor += 1;
        }
    }

    None
}

fn is_module_context(prefix: &str) -> bool {
    let prefix = prefix.trim_end();
    prefix == "import"
        || prefix.ends_with(" from")
        || prefix.ends_with("import(")
        || prefix.ends_with("require(")
}

pub(super) fn resolve_specifier(current_uri: &Url, specifier: &str) -> Option<PathBuf> {
    resolve_specifier_with(current_uri, specifier, &mut PackageRouteResolver::default())
}

fn resolve_specifier_with(
    current_uri: &Url,
    specifier: &str,
    package_routes: &mut PackageRouteResolver,
) -> Option<PathBuf> {
    let current_file = current_uri.to_file_path().ok()?;
    let current_dir = current_file.parent()?;

    if specifier.starts_with("./") || specifier.starts_with("../") {
        return resolve_file_candidate(&current_dir.join(specifier));
    }
    if is_absolute_specifier(specifier) {
        return None;
    }

    package_routes
        .resolve(
            current_dir,
            specifier,
            PackageSourceOptions::new(true, true),
        )
        .and_then(|route| route.unambiguous_source_path().cloned())
        .filter(|path| path.is_file())
}

fn is_absolute_specifier(specifier: &str) -> bool {
    if Path::new(specifier).is_absolute() || specifier.starts_with("\\\\") {
        return true;
    }
    let bytes = specifier.as_bytes();
    bytes.len() >= 3
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':'
        && matches!(bytes[2], b'/' | b'\\')
}

fn resolve_file_candidate(candidate: &Path) -> Option<PathBuf> {
    if candidate.is_file() {
        return candidate.canonicalize().ok();
    }
    let extension_mode = extension_mode(candidate);
    for extension in [
        "vue", "d.ts", "d.mts", "d.cts", "ts", "tsx", "mts", "cts", "js", "mjs", "cjs",
    ] {
        let with_extension = match extension_mode {
            ExtensionMode::Replace => candidate.with_extension(extension),
            ExtensionMode::Append => append_extension(candidate, extension),
        };
        if with_extension.is_file() {
            return with_extension.canonicalize().ok();
        }
    }
    if candidate.is_dir() {
        for basename in [
            "index.d.ts",
            "index.d.mts",
            "index.d.cts",
            "index.ts",
            "index.js",
        ] {
            let index = candidate.join(basename);
            if index.is_file() {
                return index.canonicalize().ok();
            }
        }
    }
    None
}

#[derive(Clone, Copy)]
enum ExtensionMode {
    Replace,
    Append,
}

fn extension_mode(candidate: &Path) -> ExtensionMode {
    let Some(extension) = candidate
        .extension()
        .and_then(|extension| extension.to_str())
    else {
        return ExtensionMode::Replace;
    };
    if [
        "vue", "d.ts", "d.mts", "d.cts", "ts", "tsx", "mts", "cts", "js", "jsx", "mjs", "cjs",
    ]
    .contains(&extension)
    {
        ExtensionMode::Replace
    } else {
        ExtensionMode::Append
    }
}

fn append_extension(path: &Path, extension: &str) -> PathBuf {
    path.file_name().and_then(|name| name.to_str()).map_or_else(
        || path.to_path_buf(),
        |name| path.with_file_name(cstr!("{name}.{extension}")),
    )
}
