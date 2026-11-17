use vize_l0::Span;

use super::{
    PugProfile, StyleLanguage, TemplateLang, is_css_style, is_html_template, is_preprocessor_style,
    is_pug_template, markup_extension, sfc_template_lang, style_lang, template_lang,
    template_spelling,
};

#[test]
fn template_lang_values_select_one_grammar_case_insensitively() {
    for (lang, expected) in [
        (None, TemplateLang::Html),
        (Some(""), TemplateLang::Html),
        (Some("html"), TemplateLang::Html),
        (Some("HTML"), TemplateLang::Html),
        (Some("pug"), TemplateLang::Pug(PugProfile::Pug3)),
        (Some("Pug"), TemplateLang::Pug(PugProfile::Pug3)),
        (Some("PUG"), TemplateLang::Pug(PugProfile::Pug3)),
        (Some("jade"), TemplateLang::Pug(PugProfile::Jade1)),
        (Some("Jade"), TemplateLang::Pug(PugProfile::Jade1)),
        (Some("JADE"), TemplateLang::Pug(PugProfile::Jade1)),
    ] {
        assert_eq!(template_lang(lang), Ok(expected), "{lang:?}");
        assert_eq!(is_html_template(lang), expected == TemplateLang::Html);
        assert_eq!(is_pug_template(lang), expected != TemplateLang::Html);
    }
    assert_eq!(template_spelling(Some("JADE")), Ok("jade"));
    assert_eq!(template_spelling(Some("Pug")), Ok("pug"));
    assert_eq!(template_spelling(None), Ok("html"));
}

#[test]
fn unknown_template_langs_are_unsupported_never_html() {
    for lang in [
        "haml",
        "slm",
        "ejs",
        "handlebars",
        " pug ",
        "pug ",
        "vue",
        "art",
    ] {
        let unsupported = template_lang(Some(lang)).expect_err(lang);
        assert_eq!(unsupported.authored(), lang);
        assert_eq!(unsupported.span(), None);
        assert!(!is_html_template(Some(lang)), "{lang}");
        assert!(!is_pug_template(Some(lang)), "{lang}");
    }
}

#[test]
fn style_lang_values_select_one_language_case_insensitively() {
    for (lang, expected) in [
        (None, StyleLanguage::Css),
        (Some(""), StyleLanguage::Css),
        (Some("css"), StyleLanguage::Css),
        (Some("CSS"), StyleLanguage::Css),
        (Some("postcss"), StyleLanguage::Css),
        (Some("PostCSS"), StyleLanguage::Css),
        (Some("scss"), StyleLanguage::Scss),
        (Some("SCSS"), StyleLanguage::Scss),
        (Some("sass"), StyleLanguage::Sass),
        (Some("Sass"), StyleLanguage::Sass),
        (Some("less"), StyleLanguage::Less),
        (Some("LESS"), StyleLanguage::Less),
        (Some("stylus"), StyleLanguage::Stylus),
        (Some("Stylus"), StyleLanguage::Stylus),
        (Some("styl"), StyleLanguage::Stylus),
        (Some("STYL"), StyleLanguage::Stylus),
    ] {
        assert_eq!(style_lang(lang), Ok(expected), "{lang:?}");
        assert_eq!(is_css_style(lang), expected == StyleLanguage::Css);
        assert_eq!(is_preprocessor_style(lang), expected != StyleLanguage::Css);
    }
    for lang in ["sugarss", "pcss", " css", "tailwind"] {
        assert_eq!(style_lang(Some(lang)).map_err(|u| u.authored()), Err(lang));
        assert!(!is_css_style(Some(lang)), "{lang}");
        assert!(!is_preprocessor_style(Some(lang)), "{lang}");
    }
}

#[test]
fn markup_extensions_select_the_whole_file_grammar() {
    for (extension, expected) in [
        ("html", Some(TemplateLang::Html)),
        ("htm", Some(TemplateLang::Html)),
        ("HTML", Some(TemplateLang::Html)),
        ("pug", Some(TemplateLang::Pug(PugProfile::Pug3))),
        ("jade", Some(TemplateLang::Pug(PugProfile::Jade1))),
        ("Jade", Some(TemplateLang::Pug(PugProfile::Jade1))),
        ("vue", None),
        ("tsrx", None),
        ("haml", None),
    ] {
        assert_eq!(markup_extension(extension), expected, "{extension}");
    }
}

#[test]
fn sfc_template_lang_reads_the_container_template() {
    for (source, expected) in [
        ("<template><p/></template>", Some(Ok(TemplateLang::Html))),
        (
            "<script setup>const pug = 1</script>\n<template lang=\"JADE\">p</template>",
            Some(Ok(TemplateLang::Pug(PugProfile::Jade1))),
        ),
        (
            "<template lang='pug'>p</template>",
            Some(Ok(TemplateLang::Pug(PugProfile::Pug3))),
        ),
        (
            "<template lang=pug>p</template>",
            Some(Ok(TemplateLang::Pug(PugProfile::Pug3))),
        ),
        ("<template src=\"./a.pug\" lang=\"pug\"></template>", None),
        ("<script>const lang = 'pug'</script>", None),
        // Mentions of pug outside the template's `lang` select nothing.
        (
            "<!-- lang=\"pug\" --><template><p>pug jade</p></template>",
            Some(Ok(TemplateLang::Html)),
        ),
    ] {
        assert_eq!(
            sfc_template_lang(source).map(|lang| lang.map_err(|u| u.authored())),
            expected,
            "{source}"
        );
    }
}

#[test]
fn unsupported_sfc_template_lang_carries_its_authored_span() {
    let source = "<script></script>\n<template id=\"a\" lang=\"haml\">%p</template>";
    let unsupported = sfc_template_lang(source)
        .expect("template")
        .expect_err("haml");
    assert_eq!(unsupported.authored(), "haml");
    let span = unsupported.span().expect("span");
    assert_eq!(
        source.get(span.start as usize..span.end as usize),
        Some("haml")
    );

    let unquoted = "<template lang=haml>%p</template>";
    let span = sfc_template_lang(unquoted)
        .expect("template")
        .expect_err("haml")
        .span();
    assert_eq!(span, Some(Span::new(15, 19)));
}
