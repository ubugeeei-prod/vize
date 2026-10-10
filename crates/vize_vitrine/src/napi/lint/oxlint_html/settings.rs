use serde_json::Value;
use vize_l0::{String, cstr};

use super::ProjectedSettings;

pub(super) fn project(root: &Value) -> Result<ProjectedSettings, String> {
    let mut resolved = ProjectedSettings {
        locale: "en".into(),
        help_level: "full".into(),
        preset: "incremental".into(),
    };
    let Some(settings) = root.get("settings") else {
        return Ok(resolved);
    };
    let Some(settings) = settings.as_object() else {
        return Err("/settings must be an object".into());
    };
    if settings.contains_key("patina") {
        return Err("/settings/patina alias is unqualified".into());
    }
    let Some(vize) = settings.get("vize") else {
        return Ok(resolved);
    };
    let Some(vize) = vize.as_object() else {
        return Err("/settings/vize must be an object".into());
    };
    for key in vize.keys() {
        if !matches!(
            key.as_str(),
            "locale" | "helpLevel" | "showHelp" | "preset" | "typeAware"
        ) {
            return Err(cstr!("/settings/vize/{key} is unqualified"));
        }
    }
    if let Some(locale) = vize.get("locale") {
        let Some(locale @ ("en" | "ja" | "zh")) = locale.as_str() else {
            return Err("/settings/vize/locale must be en, ja or zh".into());
        };
        resolved.locale = locale.into();
    }
    if let Some(type_aware) = vize.get("typeAware")
        && type_aware.as_bool() != Some(false)
    {
        return Err("/settings/vize/typeAware requires false".into());
    }
    if let Some(show_help) = vize.get("showHelp") {
        let Some(show_help) = show_help.as_bool() else {
            return Err("/settings/vize/showHelp must be boolean".into());
        };
        resolved.help_level = if show_help { "full" } else { "none" }.into();
    }
    if let Some(help) = vize.get("helpLevel") {
        let Some(help @ ("none" | "short" | "full")) = help.as_str() else {
            return Err("/settings/vize/helpLevel must be none, short or full".into());
        };
        // A valid authored level wins over showHelp in the existing plugin.
        resolved.help_level = help.into();
    }
    if let Some(preset) = vize.get("preset") {
        let Some(preset) = preset.as_str() else {
            return Err("/settings/vize/preset must be a string".into());
        };
        resolved.preset = normalize_preset(preset)
            .ok_or_else(|| String::from("/settings/vize/preset is unqualified"))?;
    }
    Ok(resolved)
}

fn normalize_preset(value: &str) -> Option<String> {
    let key = value
        .chars()
        .filter(|character| !matches!(character, '-' | '_') && !js_whitespace(*character))
        // Every admitted alias is ASCII; never approximate contextual Unicode
        // lowercasing for an unknown value outside that closed alias set.
        .map(|character| character.to_ascii_lowercase())
        .collect::<String>();
    let preset = match key.as_str() {
        "generalrecommended" | "happypath" | "happy" | "default" | "recommended" => {
            "general-recommended"
        }
        "essential" => "essential",
        "ecosystem" | "eco" => "ecosystem",
        "incremental" | "all" => "incremental",
        "opinionated" | "opnionated" | "strict" => "opinionated",
        "nuxt" => "nuxt",
        _ => return None,
    };
    Some(preset.into())
}

// ECMAScript \s, not Rust's broader Unicode is_whitespace predicate.
fn js_whitespace(character: char) -> bool {
    matches!(
        character,
        '\u{0009}'..='\u{000D}'
            | '\u{0020}'
            | '\u{00A0}'
            | '\u{1680}'
            | '\u{2000}'..='\u{200A}'
            | '\u{2028}'
            | '\u{2029}'
            | '\u{202F}'
            | '\u{205F}'
            | '\u{3000}'
            | '\u{FEFF}'
    )
}

pub(super) fn deny_warnings(root: &Value) -> Result<bool, String> {
    let Some(options) = root.get("options") else {
        return Ok(false);
    };
    let Some(options) = options.as_object() else {
        return Err("/options must be an object".into());
    };
    for key in ["maxWarnings", "reportUnusedDisableDirectives"] {
        if options.contains_key(key) {
            return Err(cstr!("/options/{key} is unqualified"));
        }
    }
    if options
        .get("respectEslintDisableDirectives")
        .is_some_and(|value| value.as_bool() != Some(true))
    {
        return Err("/options/respectEslintDisableDirectives requires true".into());
    }
    match options.get("denyWarnings") {
        None => Ok(false),
        Some(Value::Bool(deny)) => Ok(*deny),
        Some(_) => Err("/options/denyWarnings must be boolean".into()),
    }
}
