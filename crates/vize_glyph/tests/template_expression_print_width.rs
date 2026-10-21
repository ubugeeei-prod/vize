//! Original inputs and independent whole references for nested directive calls.
use std::{error::Error, fs, path::Path};
use vize_glyph::{FormatOptions, format_sfc};

#[test]
fn directive_calls_observe_actual_indentation_and_three_public_fixed_points()
-> Result<(), Box<dyn Error>> {
    let corpus: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/formatter-regressions/expression-print-width-7876/corpus.json"
    ))?;
    let cases = corpus["cases"].as_array().ok_or("missing cases")?;
    assert_eq!(cases.len(), 16);
    let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../../tests/_fixtures/differential/formatter-regressions/expression-print-width-7876",
    );
    for case in cases {
        let id = case["id"].as_str().ok_or("missing ID")?;
        let input = fs::read(directory.join(case["input"].as_str().ok_or("missing input")?))?;
        let expected = fs::read(directory.join(case["output"].as_str().ok_or("missing output")?))?;
        let options: FormatOptions = serde_json::from_value(case["options"].clone())?;
        let mut previous = std::string::String::from_utf8(input)?;
        for pass in 1..=3 {
            let result = format_sfc(&previous, &options)?;
            assert_eq!(
                result.changed,
                previous.as_bytes() != expected,
                "{id} pass{pass}: complete changed flag"
            );
            assert_eq!(
                result.code.as_bytes(),
                expected,
                "{id} pass{pass}: whole bytes"
            );
            previous = result.code.to_string();
        }
    }
    Ok(())
}

#[test]
fn standalone_directive_expressions_observe_their_actual_indentation() -> Result<(), Box<dyn Error>>
{
    let source = r#"<template>
  <div>
    <div>
      <div>
        <div>
          <div
            v-if="
              modeStore.get() ===
              DisplayMode.BulkUpdateOfTheSelectedRowsInTheCurrentView
            "
            v-show="
              isTheSelectedRowVisible &&
              isTheOtherRowVisibleInTheCurrentView &&
              true
            "
            :title="
              selectedRowOwner?.displayName ??
              fallbackOwnerDisplayNameForRows ??
              '-'
            "
            :data-index="
              store.progressOfTheView
                .selectedIndexOfThePaginationInTheCurrentView.value
            "
            :data-total="currentItem.primaryValue + currentItem.secondaryValue + currentItem.tertiaryValue"
            class="row"
          />
        </div>
      </div>
    </div>
  </div>
</template>
"#;
    let expected = r#"<template>
  <div>
    <div>
      <div>
        <div>
          <div
            v-if="
              modeStore.get() ===
              DisplayMode.BulkUpdateOfTheSelectedRowsInTheCurrentView
            "
            v-show="
              isTheSelectedRowVisible &&
              isTheOtherRowVisibleInTheCurrentView &&
              true
            "
            :title="
              selectedRowOwner?.displayName ??
              fallbackOwnerDisplayNameForRows ??
              '-'
            "
            :data-index="
              store.progressOfTheView
                .selectedIndexOfThePaginationInTheCurrentView.value
            "
            :data-total="
              currentItem.primaryValue +
              currentItem.secondaryValue +
              currentItem.tertiaryValue
            "
            class="row"
          />
        </div>
      </div>
    </div>
  </div>
</template>
"#;
    let options = FormatOptions {
        print_width: 80,
        sort_attributes: false,
        ..FormatOptions::default()
    };

    let first = format_sfc(source, &options)?;
    let second = format_sfc(&first.code, &options)?;

    assert_eq!(first.code.as_str(), expected);
    assert_eq!(first.code, second.code, "fmt; fmt must be a no-op");
    Ok(())
}
