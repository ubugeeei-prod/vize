//! One self-contained SFC per exact-rule defect class.
//!
//! `[[...]]` marks the expected diagnostic span. A class whose rule is not
//! `exact` or `sound` fails `--check-classes`.

pub struct Snippet {
    pub id: &'static str,
    pub rule: &'static str,
    pub source: &'static str,
}

const fn snippet(id: &'static str, rule: &'static str, source: &'static str) -> Snippet {
    Snippet { id, rule, source }
}

pub const SNIPPETS: &[Snippet] = &[
    snippet(
        "no-v-html",
        "vue/no-v-html",
        "<template>\n<div [[v-html=\"content\"]]></div>\n</template>\n",
    ),
    snippet(
        "valid-v-else",
        "vue/valid-v-else",
        "<template>\n<div v-if=\"foo\"></div><div [[v-else=\"bar\"]]></div>\n</template>\n",
    ),
    snippet(
        "no-duplicate-attributes",
        "vue/no-duplicate-attributes",
        "<template>\n<div id=\"foo\" [[id=\"bar\"]]></div>\n</template>\n",
    ),
    snippet(
        "require-v-for-key",
        "vue/require-v-for-key",
        "<template>\n<ul><li [[v-for=\"item in items\"]]>{{ item.name }}</li></ul>\n</template>\n",
    ),
    snippet(
        "no-textarea-mustache",
        "vue/no-textarea-mustache",
        "<template>\n<textarea>[[{{ message }}]]</textarea>\n</template>\n",
    ),
    snippet(
        "no-multi-spaces",
        "vue/no-multi-spaces",
        "<template>\n<div[[  ]]class=\"a\"></div>\n</template>\n",
    ),
    snippet(
        "no-autofocus",
        "a11y/no-autofocus",
        "<template>\n<input [[autofocus]]>\n</template>\n",
    ),
    snippet(
        "no-access-key",
        "a11y/no-access-key",
        "<template>\n<button [[accesskey=\"h\"]]></button>\n</template>\n",
    ),
    snippet(
        "deprecated-element",
        "html/deprecated-element",
        "<template>\n[[<center>]]text</center>\n</template>\n",
    ),
    snippet(
        "require-component-is",
        "vue/require-component-is",
        "<template>\n[[<component>]]</component>\n</template>\n",
    ),
    snippet(
        "iframe-has-title",
        "a11y/iframe-has-title",
        "<template>\n[[<iframe src=\"https://example.com\">]]</iframe>\n</template>\n",
    ),
    snippet(
        "heading-has-content",
        "a11y/heading-has-content",
        "<template>\n[[<h1>]]</h1>\n</template>\n",
    ),
    snippet(
        "no-consecutive-br",
        "html/no-consecutive-br",
        "<template>\n<p>a<br>[[<br>]]b</p>\n</template>\n",
    ),
    snippet(
        "require-datetime",
        "html/require-datetime",
        "<template>\n[[<time>]]last Tuesday</time>\n</template>\n",
    ),
    snippet(
        "no-template-key",
        "vue/no-template-key",
        "<template>\n<template [[:key=\"section\"]]><div></div></template>\n</template>\n",
    ),
    snippet(
        "no-lone-template",
        "vue/no-lone-template",
        "<template>\n<div>[[<template>]]<div>content</div></template></div>\n</template>\n",
    ),
    snippet(
        "id-duplication",
        "html/id-duplication",
        "<template>\n<div id=\"content\">first</div>\n<div [[id=\"content\"]]>second</div>\n</template>\n",
    ),
    snippet(
        "html-quotes",
        "vue/html-quotes",
        "<template>\n<div class=[['foo']]></div>\n</template>\n",
    ),
    snippet(
        "no-child-content",
        "vue/no-child-content",
        "<template>\n[[<div v-text=\"content\">]]child</div>\n</template>\n",
    ),
];
