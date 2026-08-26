use vize_glyph::{FormatOptions, format_sfc, format_template};

#[test]
fn object_v_bind_stays_between_independently_sorted_attribute_groups() {
    let source = r#"<a title="link" class="router-link" v-bind="attrs" target="_blank" rel="noopener" :href="href"></a>"#;
    let options = FormatOptions {
        print_width: 200,
        ..FormatOptions::default()
    };

    let first = format_template(source, &options).unwrap();
    let second = format_template(&first, &options).unwrap();

    assert_eq!(
        first.as_str(),
        r#"<a class="router-link" title="link" v-bind="attrs" rel="noopener" target="_blank" :href="href"></a>"#
    );
    assert_eq!(first, second);
}

#[test]
fn default_order_matches_patina_vue_attribute_order_groups() {
    // Dynamic directives are evaluation-order barriers. Static attributes
    // after the final directive can still sort safely.
    let options = FormatOptions {
        print_width: 200,
        ..FormatOptions::default()
    };

    let source =
        r#"<div class="_button" v-tooltip:dialog="tip" v-once v-show="open" id="help"></div>"#;
    let first = format_template(source, &options).unwrap();
    let second = format_template(&first, &options).unwrap();
    assert_eq!(
        first.as_str(),
        r#"<div class="_button" v-tooltip:dialog="tip" v-once v-show="open" id="help"></div>"#
    );
    assert_eq!(first, second);

    let legacy_slot = r#"<div class="box" slot="header"></div>"#;
    let first = format_template(legacy_slot, &options).unwrap();
    let second = format_template(&first, &options).unwrap();
    assert_eq!(first.as_str(), r#"<div slot="header" class="box"></div>"#);
    assert_eq!(first, second);

    let slotted = r#"<Comp :data="d" #default="{ x }"></Comp>"#;
    let first = format_template(slotted, &options).unwrap();
    let second = format_template(&first, &options).unwrap();
    assert_eq!(
        first.as_str(),
        r#"<Comp :data="d" #default="{ x }"></Comp>"#
    );
    assert_eq!(first, second);
}

#[test]
fn object_v_on_stays_between_independently_sorted_event_groups() {
    let source = r#"<button @keyup="up" @click="click" v-on="listeners" @mouseup="up" @mousedown="down"></button>"#;
    let options = FormatOptions {
        print_width: 200,
        ..FormatOptions::default()
    };

    let first = format_template(source, &options).unwrap();
    let second = format_template(&first, &options).unwrap();

    assert_eq!(
        first.as_str(),
        r#"<button @keyup="up" @click="click" v-on="listeners" @mouseup="up" @mousedown="down"></button>"#
    );
    assert_eq!(first, second);
}

#[test]
fn object_directive_modifiers_are_also_ordering_barriers() {
    let options = FormatOptions {
        print_width: 200,
        ..FormatOptions::default()
    };
    let cases = [
        (
            r#"<div title="x" v-bind.prop="attrs" id="y"></div>"#,
            r#"<div title="x" v-bind.prop="attrs" id="y"></div>"#,
        ),
        (
            r#"<div @keyup="up" v-on.stop="listeners" @click="click"></div>"#,
            r#"<div @keyup="up" v-on.stop="listeners" @click="click"></div>"#,
        ),
    ];

    for (source, expected) in cases {
        let first = format_template(source, &options).unwrap();
        let second = format_template(&first, &options).unwrap();
        assert_eq!(first.as_str(), expected);
        assert_eq!(first, second);
    }
}

#[test]
fn multiline_directive_attribute_value_is_indented_from_attribute_depth() {
    let source = r#"<span
  :class='[
rec.years.includes(y) && selectedYear === y
  ? "bg-accent border border-accent text-accent-ink"
  : rec.years.includes(y)
    ? "bg-ink border border-ink text-paper"
    : "border border-ink text-ink",
]'
  :title="y"
></span>"#;

    let options = FormatOptions::default();
    let first = format_template(source, &options).unwrap();
    let second = format_template(&first, &options).unwrap();

    assert_eq!(
        first.as_str(),
        r#"<span
  :class='[
    rec.years.includes(y) && selectedYear === y
      ? "bg-accent border border-accent text-accent-ink"
      : rec.years.includes(y)
        ? "bg-ink border border-ink text-paper"
        : "border border-ink text-ink",
  ]'
  :title="y"
></span>"#
    );
    assert_eq!(first, second);
}

#[test]
fn sfc_multiline_directive_attribute_keeps_template_indent() {
    let source = "<template>\n  <button\n    type=\"button\"\n    :class='sort === \"name-asc\" || sort === \"name-desc\"\n    ? \"bg-ink text-paper border-ink\"\n    : \"border-rule text-ink-2 hover:text-ink hover:border-ink\"'\n    @click=\"toggleNameSort\"\n  >\n    Name\n  </button>\n</template>\n";
    let options = FormatOptions::default();
    let first = format_sfc(source, &options).unwrap();
    let second = format_sfc(&first.code, &options).unwrap();

    assert_eq!(
        first.code.as_str(),
        "<template>\n  <button\n    type=\"button\"\n    :class='sort === \"name-asc\" || sort === \"name-desc\"\n      ? \"bg-ink text-paper border-ink\"\n      : \"border-rule text-ink-2 hover:text-ink hover:border-ink\"'\n    @click=\"toggleNameSort\"\n  >\n    Name\n  </button>\n</template>\n"
    );
    assert_eq!(first.code, second.code);
}

#[test]
fn sfc_multiline_directive_statement_continuation_is_anchored_like_a_ternary() {
    // #3346: the ternary continuation is re-derived from the formatted
    // expression on every pass and therefore holds still, but a statement
    // sequence was reprinted exactly as authored, so the SFC indent step
    // widened it by one level per pass (unbounded drift). Both shapes must land
    // on attribute indent + one level: the ternary continuations move 4 -> 6
    // columns, the statement continuation 8 -> 6, and neither moves again.
    let source = "<template>\n  <button\n    :class='sort === \"name-asc\" || sort === \"name-desc\"\n    ? \"bg-ink text-paper border-ink\"\n    : \"border-rule text-ink-2 hover:text-ink hover:border-ink\"'\n    @click=\"toggleNameSort(sort);\n        applySort()\"\n  >\n    Name\n  </button>\n</template>\n";
    let options = FormatOptions::default();
    let first = format_sfc(source, &options).unwrap();
    let second = format_sfc(&first.code, &options).unwrap();
    let third = format_sfc(&second.code, &options).unwrap();

    assert_eq!(
        first.code.as_str(),
        "<template>\n  <button\n    :class='sort === \"name-asc\" || sort === \"name-desc\"\n      ? \"bg-ink text-paper border-ink\"\n      : \"border-rule text-ink-2 hover:text-ink hover:border-ink\"'\n    @click=\"toggleNameSort(sort);\n      applySort()\"\n  >\n    Name\n  </button>\n</template>\n"
    );
    assert_eq!(first.code, second.code, "fmt; fmt must be a no-op");
    assert_eq!(second.code, third.code, "fmt must stay at its fixed point");
}

#[test]
fn sfc_multiline_directive_statement_continuation_keeps_relative_depth() {
    // Anchoring re-derives the continuation indentation from the value's own
    // common indent, so the nesting inside the statement sequence survives
    // instead of collapsing onto a single level.
    let source = r#"<template>
  <button
    @click="dispatch({
        id: item.id,
        kind: 'primary',
      });
      close()"
  >
    Go
  </button>
</template>
"#;
    let options = FormatOptions::default();
    let first = format_sfc(source, &options).unwrap();
    let second = format_sfc(&first.code, &options).unwrap();

    assert_eq!(first.code.as_str(), source);
    assert_eq!(first.code, second.code, "fmt; fmt must be a no-op");
}

#[test]
fn sfc_single_multiline_directive_attribute_is_idempotent() {
    let source = r#"<template>
  <label
    :style="props.reverseOrder
      ? 'grid-template-areas: \'toggle . label-text\''
      : 'grid-template-areas: \'label-text . toggle\''"
  >
  </label>
</template>
"#;
    let options = FormatOptions::default();
    let first = format_sfc(source, &options).unwrap();
    let second = format_sfc(&first.code, &options).unwrap();
    let third = format_sfc(&second.code, &options).unwrap();

    assert_eq!(first.code, second.code, "fmt; fmt must be a no-op");
    assert_eq!(second.code, third.code, "fmt must stay at its fixed point");
    assert!(
        first.code.contains("\n    :style="),
        "single multiline attribute should stay on its own line:\n{}",
        first.code
    );
}

#[test]
fn sfc_verbatim_multiline_directive_attribute_is_idempotent() {
    let source = r#"<template>
  <QBtn
    @click.stop="
      selectWord(key);
      editWord();
    "
  />
</template>
"#;
    let options = FormatOptions::default();
    let first = format_sfc(source, &options).unwrap();
    let second = format_sfc(&first.code, &options).unwrap();
    let third = format_sfc(&second.code, &options).unwrap();

    assert_eq!(first.code, second.code, "fmt; fmt must be a no-op");
    assert_eq!(second.code, third.code, "fmt must stay at its fixed point");
}

#[test]
fn sfc_multiline_v_for_collection_is_idempotent() {
    let source = r#"<template>
  <template
    v-for="(engineId, engineIndex) in sortedEngineInfos.map(
      (engineInfo) => engineInfo.uuid,
    )"
    :key="engineIndex"
  >
    <span>{{ engineId }}</span>
  </template>
</template>
"#;
    let options = FormatOptions::default();
    let first = format_sfc(source, &options).unwrap();
    let second = format_sfc(&first.code, &options).unwrap();
    let third = format_sfc(&second.code, &options).unwrap();

    assert_eq!(first.code, second.code, "fmt; fmt must be a no-op");
    assert_eq!(second.code, third.code, "fmt must stay at its fixed point");
}

#[test]
fn sfc_multiline_template_literal_directive_attribute_is_idempotent() {
    let source = r#"<template>
  <NuxtLink
    :class="isSmallScreen
      ? `
        w-full
        px5 sm:mxa
      `
      : `
        w-fit rounded-3
        px2 mx3 sm:mxa
      `"
  />
</template>
"#;
    let options = FormatOptions::default();
    let first = format_sfc(source, &options).unwrap();
    let second = format_sfc(&first.code, &options).unwrap();
    let third = format_sfc(&second.code, &options).unwrap();

    assert_eq!(first.code, second.code, "fmt; fmt must be a no-op");
    assert_eq!(second.code, third.code, "fmt must stay at its fixed point");
}

#[test]
fn sfc_complex_nuxt_template_converges_with_unsorted_attributes() {
    let source = r#"<template>
  <HeaderTop
    v-if="studyInfo && currentQuestion"
    :breadcrumbs="[
      { label: purpose.name, to: `/purposes/${purpose.id}` },
      { label: studyInfo.title, to: `/purposes/${purpose.id}/studies/${studyInfo.id}` },
    ]"
    :class="[
      isOpen ? 'bg-paper border-rule' : 'bg-mute border-transparent',
      currentQuestion.status === 'answered'
        ? 'text-success'
        : currentQuestion.status === 'skipped'
          ? 'text-warning'
          : 'text-ink',
    ]"
    :progress="{
      current: questionIndex + 1,
      total: questions.length,
      label: `${questionIndex + 1}/${questions.length}`,
    }"
    @click:next="() => moveQuestion({
      purposeId: purpose.id,
      studyInfoId: studyInfo.id,
      questionId: currentQuestion.id,
    })"
  >
    <template #actions="{ disabled, submit }">
      <button
        :disabled="disabled || loading"
        @click="submit({
          answerStatus: currentQuestion.status,
          selectedIds: selectedChoices.map((choice) => choice.id),
        })"
      >
        Next
      </button>
    </template>
  </HeaderTop>
</template>
"#;
    let options = FormatOptions {
        print_width: 120,
        sort_attributes: false,
        ..FormatOptions::default()
    };
    let first = format_sfc(source, &options).unwrap();
    let second = format_sfc(&first.code, &options).unwrap();
    let third = format_sfc(&second.code, &options).unwrap();

    assert_eq!(first.code, second.code, "fmt; fmt must be a no-op");
    assert_eq!(second.code, third.code, "fmt must stay at its fixed point");
}
