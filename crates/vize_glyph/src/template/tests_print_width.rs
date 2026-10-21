use super::{FormatOptions, format_template_content};

#[test]
fn test_print_width_triggers_multiline() {
    // Very narrow print_width should trigger multiline
    let source = r#"<div class="container" id="main" title="tooltip"></div>"#;
    let options = FormatOptions {
        print_width: 30,
        ..FormatOptions::default()
    };
    let result = format_template_content(source, &options).unwrap();

    let lines: Vec<&str> = result.lines().collect();
    assert!(
        lines.len() > 2,
        "Narrow print_width should trigger multiline attributes"
    );
}

#[test]
fn single_long_attribute_wraps_at_print_width() {
    // #7236: one attribute used to ignore print width and was joined back
    // onto the tag line. The block class is longer than 80 on its own; the
    // inline class is exactly 80 at depth 0 (`<span class="…">`) and over 80
    // once indented under a parent.
    let block_class =
        "a-very-long-block-element-class-name-that-alone-is-longer-than-the-print-width";
    let inline_class = "a-long-inline-element-class-name-that-is-close-to-the-print-width";
    let options = FormatOptions {
        print_width: 80,
        ..FormatOptions::default()
    };

    let inline = format!(r#"<div class="{block_class}">text</div>"#);
    assert_eq!(
        format_template_content(&inline, &options).unwrap().as_str(),
        format!("<div\n  class=\"{block_class}\"\n>text</div>")
    );

    let already_wrapped = format!("<div\n  class=\"{block_class}\"\n>\n  text\n</div>\n");
    let wrapped = format_template_content(&already_wrapped, &options).unwrap();
    assert_eq!(
        wrapped.as_str(),
        format!("<div\n  class=\"{block_class}\"\n>\n  text\n</div>")
    );
    assert_eq!(
        format_template_content(&wrapped, &options).unwrap(),
        wrapped,
        "wrapping a single long attribute must be stable"
    );

    let fits = format!(r#"<span class="{inline_class}">x</span>"#);
    assert_eq!(
        format_template_content(&fits, &options).unwrap().as_str(),
        format!("<span class=\"{inline_class}\"\n>x</span>"),
        "the complete hugged element contributes to the width decision"
    );

    let nested = format!(
        "<div>\n  <span class=\"{inline_class}\">{{{{ lap }}}}</span>\n  <div class=\"ok\">text</div>\n</div>\n"
    );
    let nested_formatted = format_template_content(&nested, &options).unwrap();
    assert_eq!(
        nested_formatted.as_str(),
        format!(
            "<div>\n  <span\n    class=\"{inline_class}\"\n  >{{{{ lap }}}}</span>\n  <div class=\"ok\">text</div>\n</div>"
        )
    );
}

#[test]
fn hugged_inline_children_contribute_to_opening_tag_width() {
    let options = FormatOptions {
        print_width: 80,
        sort_attributes: false,
        ..FormatOptions::default()
    };
    let source = r#"<div>
  <span class="status-label" :title="statusText">{{ statusText }} of {{ totalCount }} entries</span>
  <div class="message-body" :data-kind="kind">A plain sentence that is long enough to overflow.</div>
</div>"#;
    let expected = r#"<div>
  <span class="status-label" :title="statusText"
  >{{ statusText }} of {{ totalCount }} entries</span>
  <div class="message-body" :data-kind="kind"
  >A plain sentence that is long enough to overflow.</div>
</div>"#;

    let formatted = format_template_content(source, &options).unwrap();
    assert_eq!(formatted.as_str(), expected);
    assert!(formatted.lines().all(|line| line.len() <= 80));
    assert_eq!(
        format_template_content(&formatted, &options).unwrap(),
        formatted,
        "hugged text and mixed interpolation layout must be stable"
    );
}

#[test]
fn authored_multiline_mixed_child_uses_its_formatted_width_on_the_first_pass() {
    let options = FormatOptions {
        print_width: 80,
        ..FormatOptions::default()
    };
    let source = r#"<div>
  <span class="variant-count"
    >{{ parsed.variants.length }} variant{{
      parsed.variants.length != 1 ? "s" : ""
    }}</span
  >
</div>"#;

    let formatted = format_template_content(source, &options).unwrap();
    assert_eq!(
        format_template_content(&formatted, &options).unwrap(),
        formatted
    );
}

#[test]
fn authored_hugged_text_wrap_does_not_rejoin_past_print_width() {
    let options = FormatOptions {
        print_width: 100,
        sort_attributes: false,
        ..FormatOptions::default()
    };
    let source = r#"<div class="toolbar">
  <template v-if="isEditing">
    <MyButton color="primary" :disabled="!isReorderInputReady" @click="openReorderConfirmDialog"
      >Finish reordering</MyButton
    >
  </template>
</div>"#;
    let expected = r#"<div class="toolbar">
  <template v-if="isEditing">
    <MyButton color="primary" :disabled="!isReorderInputReady" @click="openReorderConfirmDialog"
    >Finish reordering</MyButton>
  </template>
</div>"#;

    let formatted = format_template_content(source, &options).unwrap();
    assert_eq!(formatted.as_str(), expected);
    assert!(formatted.lines().all(|line| line.len() <= 100));
    assert_eq!(
        format_template_content(&formatted, &options).unwrap(),
        formatted
    );
}

#[test]
fn adjacent_inline_template_children_each_respect_print_width() {
    let options = FormatOptions {
        print_width: 90,
        sort_attributes: false,
        ..FormatOptions::default()
    };
    let source = r#"<div>
  <p>
    Assigned {{ summary.assigned }} items<template v-if="summary.released > 0">, released {{ summary.released }} items</template><template v-if="summary.unchanged > 0">, unchanged {{ summary.unchanged }} items</template>
  </p>
</div>"#;
    let expected = r#"<div>
  <p>
    Assigned {{ summary.assigned }} items<template v-if="summary.released > 0"
    >, released {{ summary.released }} items</template><template
      v-if="summary.unchanged > 0"
    >, unchanged {{ summary.unchanged }} items</template>
  </p>
</div>"#;

    let formatted = format_template_content(source, &options).unwrap();
    assert_eq!(formatted.as_str(), expected);
    assert_eq!(
        format_template_content(&formatted, &options).unwrap(),
        formatted
    );
}

#[test]
fn hugged_multiline_interpolation_with_trailing_text_uses_parent_depth() {
    let options = FormatOptions {
        print_width: 90,
        ..FormatOptions::default()
    };
    let source = r#"<div>
  <div>
    <span class="my-very-long-component-name__an-element-with-a-long-name">{{
      progressPercentages.value.firstLapOfTheCurrentSelectionInTheView
    }}%</span>
  </div>
</div>"#;
    let expected = r#"<div>
  <div>
    <span class="my-very-long-component-name__an-element-with-a-long-name">{{
      progressPercentages.value.firstLapOfTheCurrentSelectionInTheView
    }}%</span>
  </div>
</div>"#;

    let formatted = format_template_content(source, &options).unwrap();
    assert_eq!(formatted.as_str(), expected);
    assert_eq!(
        format_template_content(&formatted, &options).unwrap(),
        formatted
    );
}

#[test]
fn newly_wrapped_hugged_interpolation_with_trailing_text_is_stable() {
    let options = FormatOptions {
        print_width: 40,
        ..FormatOptions::default()
    };
    let source =
        r#"<div><h3>{{ CODE_OUTPUT_LABELS[codeOutputTarget] }} Compilation Error</h3></div>"#;

    let formatted = format_template_content(source, &options).unwrap();
    assert_eq!(
        format_template_content(&formatted, &options).unwrap(),
        formatted
    );
}

#[test]
fn text_hugged_interpolations_respect_print_width() {
    let options = FormatOptions {
        print_width: 100,
        ..FormatOptions::default()
    };
    let source = r#"<div class="card">
  <section class="panel">
    <p class="tag">
      &lt;{{
        kind === "primary" ? "my-primary-widget-element-name" : "my-secondary-widget-name"
      }}&gt;
    </p>
    <p class="tag">
      Name: {{ kind === "primary" ? "my-primary-widget-element-name" : "my-secondary-widget-name-long" }}
    </p>
  </section>
</div>"#;

    let expected = r#"<div class="card">
  <section class="panel">
    <p class="tag">
      &lt;{{
        kind === "primary" ? "my-primary-widget-element-name" : "my-secondary-widget-name"
      }}&gt;
    </p>
    <p class="tag">
      Name: {{
        kind === "primary" ? "my-primary-widget-element-name" : "my-secondary-widget-name-long"
      }}
    </p>
  </section>
</div>"#;

    let formatted = format_template_content(source, &options).unwrap();
    assert_eq!(formatted.as_str(), expected);
    assert!(
        formatted.lines().all(|line| line.len() <= 100),
        "mixed text/interpolation lines must respect print_width"
    );
    assert_eq!(
        format_template_content(&formatted, &options).unwrap(),
        formatted,
        "the wrapped layout must be stable"
    );

    let continued = r#"<div>
  <b>x</b> Name: {{ kind === "primary" ? "my-primary-widget-element-name" : "my-secondary-widget-name-long" }}
</div>"#;
    let continued_expected = r#"<div>
  <b>x</b> Name: {{
    kind === "primary" ? "my-primary-widget-element-name" : "my-secondary-widget-name-long"
  }}
</div>"#;
    let continued_formatted = format_template_content(continued, &options).unwrap();
    assert_eq!(continued_formatted.as_str(), continued_expected);
    assert!(continued_formatted.lines().all(|line| line.len() <= 100));
    assert_eq!(
        format_template_content(&continued_formatted, &options).unwrap(),
        continued_formatted,
        "continued mixed text layout must be stable"
    );
}
