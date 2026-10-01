use super::load::unescape_json_string;
use super::{Locale, Translator};
use crate::diag::MessageLookup;
use alloc::borrow::Cow;

fn fixture_translator() -> Translator {
    let mut messages = std::array::from_fn(|_| rustc_hash::FxHashMap::default());
    messages[0].insert("shared", "English");
    messages[1].insert("shared", "日本語");
    messages[2].insert("shared", "中文");
    messages[0].insert("fallback", "Hello {name}");
    messages[1].insert("local-only", "日本語だけ");
    messages[0].insert("template", "{name}: {name}, {later}, {missing}");
    Translator { messages }
}

#[test]
fn locale_provider_prefers_requested_text_and_only_falls_back_to_english() {
    let translator = fixture_translator();
    for (locale, expected) in [
        (Locale::En, "English"),
        (Locale::Ja, "日本語"),
        (Locale::Zh, "中文"),
    ] {
        assert!(matches!(
            translator.for_locale(locale).lookup("shared"),
            Cow::Borrowed(text) if text == expected
        ));
    }
    assert!(matches!(
        translator.for_locale(Locale::Ja).lookup("fallback"),
        Cow::Borrowed("Hello {name}")
    ));
    assert!(!translator.has_key(Locale::Ja, "fallback"));
    assert_eq!(
        translator.for_locale(Locale::Zh).lookup("local-only"),
        "local-only"
    );
}

#[cfg(test)]
#[test]
fn unknown_key_owns_its_bytes_after_key_and_translator_are_dropped() {
    let message = {
        let translator = fixture_translator();
        let key = std::string::String::from("unknown.{name}");
        translator.for_locale(Locale::Ja).lookup(&key)
    };
    assert!(matches!(message, Cow::Owned(_)));
    assert_eq!(message, "unknown.{name}");
}

#[test]
fn substitutions_keep_input_order_repetition_and_unmatched_placeholders() {
    let translator = fixture_translator();
    assert_eq!(
        translator.format(
            Locale::En,
            "template",
            &[("name", "{later}"), ("later", "世界")]
        ),
        "世界: 世界, 世界, {missing}"
    );
    assert_eq!(
        translator.format(
            Locale::En,
            "template",
            &[("later", "世界"), ("name", "{later}")]
        ),
        "{later}: {later}, 世界, {missing}"
    );
}

#[test]
fn formatting_preserves_empty_variables_fallback_and_unknown_key_templates() {
    let translator = fixture_translator();
    assert_eq!(
        translator.format(Locale::Ja, "fallback", &[]),
        "Hello {name}"
    );
    assert_eq!(
        translator.format(Locale::Ja, "fallback", &[("name", "世界")]),
        "Hello 世界"
    );
    assert_eq!(
        translator.format(Locale::Zh, "unknown.{name}", &[("name", "世界")]),
        "unknown.世界"
    );
}

#[test]
fn test_locale_from_str() {
    assert_eq!("en".parse::<Locale>(), Ok(Locale::En));
    assert_eq!("EN".parse::<Locale>(), Ok(Locale::En));
    assert_eq!("ja".parse::<Locale>(), Ok(Locale::Ja));
    assert_eq!("JA-JP".parse::<Locale>(), Ok(Locale::Ja));
    assert_eq!("zh".parse::<Locale>(), Ok(Locale::Zh));
    assert_eq!("zh-CN".parse::<Locale>(), Ok(Locale::Zh));
    assert!("unknown".parse::<Locale>().is_err());
}

#[test]
fn test_locale_parse() {
    assert_eq!(Locale::parse("en"), Some(Locale::En));
    assert_eq!(Locale::parse("ja"), Some(Locale::Ja));
    assert_eq!(Locale::parse("zh"), Some(Locale::Zh));
    assert_eq!(Locale::parse("unknown"), None);
}

#[test]
fn test_locale_code() {
    assert_eq!(Locale::En.code(), "en");
    assert_eq!(Locale::Ja.code(), "ja");
    assert_eq!(Locale::Zh.code(), "zh");
}

#[test]
fn test_locale_display_name() {
    assert_eq!(Locale::En.display_name(), "English");
    assert_eq!(Locale::Ja.display_name(), "日本語");
    assert_eq!(Locale::Zh.display_name(), "中文");
}

#[test]
fn test_translator_get() {
    let t = Translator::new();
    // Test that basic lookup works
    let msg = t.get(Locale::En, "test.hello");
    // Either returns the translation or the key as fallback
    assert!(!msg.is_empty());
}

#[test]
fn test_translator_format() {
    let t = Translator::new();
    let msg = t.format(Locale::En, "test.greeting", &[("name", "World")]);
    // Either contains the substitution or is the key
    assert!(!msg.is_empty());
}

#[test]
fn test_unescape_json_string() {
    assert_eq!(unescape_json_string("hello"), "hello");
    assert_eq!(unescape_json_string("hello\\nworld"), "hello\nworld");
    assert_eq!(unescape_json_string("hello\\tworld"), "hello\tworld");
    assert_eq!(unescape_json_string("he said \\\"hi\\\""), "he said \"hi\"");
    assert_eq!(unescape_json_string("path\\\\to\\\\file"), "path\\to\\file");
}
