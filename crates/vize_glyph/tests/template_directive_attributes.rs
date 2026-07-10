use vize_glyph::{FormatOptions, format_sfc, format_template};

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
