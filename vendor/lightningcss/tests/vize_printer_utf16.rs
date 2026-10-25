//! Independent writer-event and real stylesheet source-map coordinates.
#![cfg(feature = "sourcemap")]

use lightningcss::{
  css_modules::{Config, Pattern},
  printer::{Printer, PrinterOptions},
  rules::Location,
  stylesheet::{ParserOptions, StyleSheet},
};
use parcel_sourcemap::SourceMap;

#[test]
fn actual_writer_events_count_utf16_only_when_mapping() {
  let mut code = String::new();
  let mut map = SourceMap::new("/");
  map.add_source("input.css");
  {
    let mut printer = Printer::new(
      &mut code,
      PrinterOptions {
        source_map: Some(&mut map),
        ..Default::default()
      },
    );
    printer.write_str("雪😀").unwrap();
    printer.add_mapping(Location {
      source_index: 0,
      line: 7,
      column: 9,
    });
    std::fmt::Write::write_str(&mut printer, "雪😀").unwrap();
    printer.add_mapping(Location {
      source_index: 0,
      line: 8,
      column: 10,
    });
    printer.write_char('😀').unwrap();
    printer.add_mapping(Location {
      source_index: 0,
      line: 9,
      column: 11,
    });
    printer.write_str_with_newlines("/*!雪😀\r\n*/雪😀").unwrap();
    printer.add_mapping(Location {
      source_index: 0,
      line: 10,
      column: 12,
    });
  }
  assert_eq!(code, "雪😀雪😀😀/*!雪😀\r\n*/雪😀");
  let actual: Vec<_> = map
    .get_mappings()
    .into_iter()
    .map(|mapping| {
      let origin = mapping.original.unwrap();
      (
        mapping.generated_line,
        mapping.generated_column,
        origin.original_line,
        origin.original_column,
      )
    })
    .collect();
  assert_eq!(actual, [(0, 3, 7, 8), (0, 6, 8, 9), (0, 8, 9, 10), (1, 5, 10, 11)]);
}

#[test]
fn serialized_identifiers_modules_and_comments_keep_actual_generated_columns() {
  for modules in [false, true] {
    let source = "/*!雪😀\r\n*/.\\31 a{content:'雪😀'}.终{opacity:.5}";
    let mut map = SourceMap::new("/");
    map.add_source("input.css");
    let stylesheet = StyleSheet::parse(
      source,
      ParserOptions {
        css_modules: modules.then(|| Config {
          pattern: Pattern::parse("[local]").unwrap(),
          ..Default::default()
        }),
        ..Default::default()
      },
    )
    .unwrap();
    let plain = stylesheet
      .to_css(PrinterOptions {
        minify: true,
        ..Default::default()
      })
      .unwrap();
    let mapped = stylesheet
      .to_css(PrinterOptions {
        minify: true,
        source_map: Some(&mut map),
        ..Default::default()
      })
      .unwrap();
    assert_eq!(mapped.code, plain.code);
    assert_eq!(mapped.exports, plain.exports);
    assert_eq!(mapped.code, "/*!雪😀\r\n*/\n.\\31 a{content:\"雪😀\"}.终{opacity:.5}");
    let actual: Vec<_> = map
      .get_mappings()
      .into_iter()
      .map(|mapping| {
        let origin = mapping.original.unwrap();
        (
          mapping.generated_line,
          mapping.generated_column,
          origin.original_line,
          origin.original_column,
        )
      })
      .collect();
    // The escaped identifier and astral string each precede the second rule.
    // These literal expected columns count physical serialized UTF16 units.
    assert_eq!(actual, [(2, 0, 1, 2), (2, 21, 1, 23)]);
  }
}

#[test]
fn module_dashed_identifier_writes_preserve_their_real_utf16_width() {
  let source = ".a{--雪😀:1;color:var(--雪😀)}.b{opacity:.5}";
  let stylesheet = StyleSheet::parse(
    source,
    ParserOptions {
      css_modules: Some(Config {
        pattern: Pattern::parse("[local]").unwrap(),
        dashed_idents: true,
        ..Default::default()
      }),
      ..Default::default()
    },
  )
  .unwrap();
  let mut map = SourceMap::new("/");
  map.add_source("input.css");
  let plain = stylesheet
    .to_css(PrinterOptions {
      minify: true,
      ..Default::default()
    })
    .unwrap();
  let mapped = stylesheet
    .to_css(PrinterOptions {
      minify: true,
      source_map: Some(&mut map),
      ..Default::default()
    })
    .unwrap();
  assert_eq!(mapped.code, source);
  assert_eq!(mapped.code, plain.code);
  assert_eq!(mapped.exports, plain.exports);
  let actual: Vec<_> = map
    .get_mappings()
    .into_iter()
    .map(|mapping| {
      let origin = mapping.original.unwrap();
      (
        mapping.generated_line,
        mapping.generated_column,
        origin.original_line,
        origin.original_column,
      )
    })
    .collect();
  assert_eq!(actual, [(0, 0, 0, 0), (0, 28, 0, 28)]);
}
