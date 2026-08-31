use std::{fs, path::Path};

pub(in crate::commands::check::runner) fn is_suppressed_false_positive(
    diagnostic: &vize_canon::BatchDiagnostic,
) -> bool {
    is_nitro_import_meta_conflict(diagnostic)
        || is_vue_wildcard_component_duplicate(diagnostic)
        || is_nuxt_bridge_injected_property_duplicate(diagnostic)
        || is_vue_expect_error_suppressed(diagnostic)
        || is_native_truthiness_false_positive(diagnostic)
        || is_corsa_recursive_discriminant_array_false_positive(diagnostic)
}

fn is_nitro_import_meta_conflict(diagnostic: &vize_canon::BatchDiagnostic) -> bool {
    diagnostic.code == Some(2320)
        && diagnostic
            .message
            .contains("Interface 'ImportMeta' cannot simultaneously extend types")
        && diagnostic.message.contains("NitroStaticBuildFlags")
        && diagnostic.message.contains("NitroImportMeta")
}

fn is_vue_wildcard_component_duplicate(diagnostic: &vize_canon::BatchDiagnostic) -> bool {
    diagnostic.code == Some(2300)
        && diagnostic
            .message
            .contains("Duplicate identifier 'component'")
        && declaration_source_contains(
            &diagnostic.file,
            &["declare module \"*.vue\"", "declare module '*.vue'"],
        )
}

fn is_nuxt_bridge_injected_property_duplicate(diagnostic: &vize_canon::BatchDiagnostic) -> bool {
    diagnostic.code == Some(2717)
        && diagnostic
            .message
            .contains("Subsequent property declarations must have the same type")
        && diagnostic.message.contains("Property '$")
        && diagnostic.message.contains("must be of type 'any'")
        && declaration_source_contains(
            &diagnostic.file,
            &[
                "declare module \"@nuxt/bridge-schema\"",
                "declare module '@nuxt/bridge-schema'",
            ],
        )
}

fn declaration_source_contains(path: &Path, needles: &[&str]) -> bool {
    if path
        .file_name()
        .and_then(|name| name.to_str())
        .is_none_or(|name| {
            !name.ends_with(".d.ts") && !name.ends_with(".d.mts") && !name.ends_with(".d.cts")
        })
    {
        return false;
    }
    let Ok(source) = fs::read_to_string(path) else {
        return false;
    };
    needles.iter().any(|needle| source.contains(needle))
}

fn is_vue_expect_error_suppressed(diagnostic: &vize_canon::BatchDiagnostic) -> bool {
    if diagnostic
        .file
        .extension()
        .and_then(|extension| extension.to_str())
        .is_none_or(|extension| extension != "vue")
    {
        return false;
    }
    let Ok(source) = fs::read_to_string(&diagnostic.file) else {
        return false;
    };
    let lines = source.lines().collect::<Vec<_>>();
    let mut line = diagnostic.line as usize;
    while line > 0 {
        line -= 1;
        let trimmed = lines.get(line).map_or("", |line| line.trim());
        if trimmed.is_empty() {
            continue;
        }
        return trimmed.contains("@vue-expect-error");
    }
    false
}

fn is_native_truthiness_false_positive(diagnostic: &vize_canon::BatchDiagnostic) -> bool {
    diagnostic.code == Some(2801)
        && diagnostic.block_type == Some(vize_canon::SfcBlockType::Template)
        && diagnostic
            .file
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension == "vue")
        && diagnostic
            .message
            .contains("This condition will always return true since this")
        && diagnostic.message.contains("is always defined")
}

fn is_corsa_recursive_discriminant_array_false_positive(
    diagnostic: &vize_canon::BatchDiagnostic,
) -> bool {
    if diagnostic.code != Some(2345)
        || diagnostic.block_type.is_some()
        || !is_typescript_source_file(&diagnostic.file)
    {
        return false;
    }

    let Some((array_element, recursive_property)) =
        recursive_discriminant_array_target(&diagnostic.message)
    else {
        return false;
    };
    source_declares_recursive_array_target(&diagnostic.file, array_element, recursive_property)
}

fn is_typescript_source_file(path: &Path) -> bool {
    let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    if file_name.ends_with(".d.ts")
        || file_name.ends_with(".d.mts")
        || file_name.ends_with(".d.cts")
    {
        return false;
    }

    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| matches!(extension, "ts" | "tsx" | "mts" | "cts"))
}

fn recursive_discriminant_array_target(message: &str) -> Option<(&str, &str)> {
    if !message.contains("is not assignable to parameter of type '")
        || !message.contains("Type '{ __typename?: ")
        || !message.contains(" }' is missing the following properties from type '")
    {
        return None;
    }

    let array_element = text_between(message, "parameter of type '", "[]'")?;
    let recursive_property = text_between(
        message,
        "Types of property '",
        "' are incompatible.\nType '",
    )?;
    if array_element.is_empty() || recursive_property.is_empty() {
        return None;
    }
    Some((array_element, recursive_property))
}

fn text_between<'a>(text: &'a str, start: &str, end: &str) -> Option<&'a str> {
    let (_, after_start) = text.split_once(start)?;
    let (value, _) = after_start.split_once(end)?;
    Some(value)
}

fn source_declares_recursive_array_target(
    path: &Path,
    array_element: &str,
    recursive_property: &str,
) -> bool {
    let Ok(source) = fs::read_to_string(path) else {
        return false;
    };

    let alias = format!("type {array_element} =");
    let array_property = format!("{recursive_property}: {array_element}[]");
    let generic_property = format!("{recursive_property}: Array<{array_element}>");
    source.contains(&alias)
        && (source.contains(&array_property) || source.contains(&generic_property))
}

#[cfg(test)]
#[path = "suppressions_tests.rs"]
mod tests;
