//! Complete published alpha72 remapping oracle plus an authored physical @import contract.
use lightningcss::{
  bundler::{Bundler, FileProvider, ResolveResult, SourceProvider},
  printer::PrinterOptions,
  stylesheet::{MinifyOptions, ParserOptions},
};
use parcel_sourcemap::{Mapping, OriginalLocation, SourceMap};
use std::{
  collections::HashMap,
  fs, io,
  path::{Path, PathBuf},
};

const ORIGINAL_IMPORTED_CSS: &str = r###".imported {
      content: "yay, file support!";
    }

    .selector {
      margin: 1em;
      background-color: #f60;
    }

    .selector .nested {
      margin: 0.5em;
    }

    /*# sourceMappingURL=data:application/json;base64,ewoJInZlcnNpb24iOiAzLAoJInNvdXJjZVJvb3QiOiAicm9vdCIsCgkiZmlsZSI6ICJzdGRvdXQiLAoJInNvdXJjZXMiOiBbCgkJInN0ZGluIiwKCQkic2Fzcy9fdmFyaWFibGVzLnNjc3MiLAoJCSJzYXNzL19kZW1vLnNjc3MiCgldLAoJInNvdXJjZXNDb250ZW50IjogWwoJCSJAaW1wb3J0IFwiX3ZhcmlhYmxlc1wiO1xuQGltcG9ydCBcIl9kZW1vXCI7XG5cbi5zZWxlY3RvciB7XG4gIG1hcmdpbjogJHNpemU7XG4gIGJhY2tncm91bmQtY29sb3I6ICRicmFuZENvbG9yO1xuXG4gIC5uZXN0ZWQge1xuICAgIG1hcmdpbjogJHNpemUgLyAyO1xuICB9XG59IiwKCQkiJGJyYW5kQ29sb3I6ICNmNjA7XG4kc2l6ZTogMWVtOyIsCgkJIi5pbXBvcnRlZCB7XG4gIGNvbnRlbnQ6IFwieWF5LCBmaWxlIHN1cHBvcnQhXCI7XG59IgoJXSwKCSJtYXBwaW5ncyI6ICJBRUFBLFNBQVMsQ0FBQztFQUNSLE9BQU8sRUFBRSxvQkFBcUI7Q0FDL0I7O0FGQ0QsU0FBUyxDQUFDO0VBQ1IsTUFBTSxFQ0hELEdBQUc7RURJUixnQkFBZ0IsRUNMTCxJQUFJO0NEVWhCOztBQVBELFNBQVMsQ0FJUCxPQUFPLENBQUM7RUFDTixNQUFNLEVDUEgsS0FBRztDRFFQIiwKCSJuYW1lcyI6IFtdCn0= */"###;

const ORIGINAL_ENTRY_CSS: &str = r###"
        @import "/b.css";
        .a { color: red; }
      "###;

const ORIGINAL_EXPECTED_MAP: &str = r###"{"version":3,"sourceRoot":null,"mappings":"ACAA,uCCGA,2CAAA,8BFDQ","sources":["a.css","sass/_demo.scss","stdin"],"sourcesContent":["\n        @import \"/b.css\";\n        .a { color: red; }\n      ",".imported {\n  content: \"yay, file support!\";\n}","@import \"_variables\";\n@import \"_demo\";\n\n.selector {\n  margin: $size;\n  background-color: $brandColor;\n\n  .nested {\n    margin: $size / 2;\n  }\n}"],"names":[]}"###;

struct WholeFixtureProvider {
  files: HashMap<PathBuf, String>,
}
impl SourceProvider for WholeFixtureProvider {
  type Error = io::Error;
  fn read<'a>(&'a self, file: &Path) -> Result<&'a str, io::Error> {
    self
      .files
      .get(file)
      .map(String::as_str)
      .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, format!("fixture {}", file.display())))
  }
  fn resolve(&self, specifier: &str, originating_file: &Path) -> Result<ResolveResult, io::Error> {
    Ok(originating_file.with_file_name(specifier).into())
  }
}

#[test]
fn complete_original_input_map_remapping() {
  let provider = WholeFixtureProvider {
    files: [
      (PathBuf::from("/a.css"), ORIGINAL_ENTRY_CSS.to_owned()),
      (PathBuf::from("/b.css"), ORIGINAL_IMPORTED_CSS.to_owned()),
    ]
    .iter()
    .cloned()
    .collect(),
  };
  let mut map = SourceMap::new("/");
  let mut bundler = Bundler::new(&provider, Some(&mut map), ParserOptions::default());
  let mut stylesheet = bundler.bundle(Path::new("/a.css")).unwrap();
  stylesheet.minify(MinifyOptions::default()).unwrap();
  let result = stylesheet
    .to_css(PrinterOptions {
      source_map: Some(&mut map),
      minify: true,
      ..Default::default()
    })
    .unwrap();
  assert!(!result.code.is_empty());
  // The full pre-existing published fixture expected string is immutable;
  // no candidate-generated snapshot or selected map field replaces it.
  assert_eq!(map.to_json(None).unwrap(), ORIGINAL_EXPECTED_MAP);
  let mut decoded = SourceMap::from_json("/", ORIGINAL_EXPECTED_MAP).unwrap();
  assert_eq!(decoded.to_json(None).unwrap(), ORIGINAL_EXPECTED_MAP);
  assert_eq!(decoded.get_sources(), map.get_sources());
  assert_eq!(decoded.get_sources_content(), map.get_sources_content());
  assert_eq!(decoded.get_names(), map.get_names());
  assert_eq!(decoded.get_mappings(), map.get_mappings());
}

#[test]
fn complete_physical_import_code_and_map() {
  let directory = assert_fs::TempDir::new().unwrap();
  let entry = "@import \"b.css\";\n.a { opacity: 1 }\n";
  let imported = ".b { opacity: .5 }\n";
  fs::write(directory.path().join("a.css"), entry).unwrap();
  fs::write(directory.path().join("b.css"), imported).unwrap();
  let mut map = SourceMap::new(directory.path().to_str().unwrap());
  let provider = FileProvider::new();
  let mut bundler = Bundler::new(&provider, Some(&mut map), ParserOptions::default());
  let mut stylesheet = bundler.bundle(&directory.path().join("a.css")).unwrap();
  stylesheet.minify(MinifyOptions::default()).unwrap();
  let result = stylesheet
    .to_css(PrinterOptions {
      source_map: Some(&mut map),
      minify: true,
      ..Default::default()
    })
    .unwrap();
  assert_eq!(result.code, ".b{opacity:.5}.a{opacity:1}");
  assert!(result.exports.is_none());
  assert!(result.references.is_none());
  assert!(result.dependencies.is_none());
  assert_eq!(map.get_sources(), &vec!["a.css".to_owned(), "b.css".to_owned()]);
  assert_eq!(map.get_sources_content(), &vec![entry.to_owned(), imported.to_owned()]);
  assert!(map.get_names().is_empty());
  assert_eq!(
    map.get_mappings(),
    vec![
      Mapping {
        generated_line: 0,
        generated_column: 0,
        original: Some(OriginalLocation::new(0, 0, 1, None))
      },
      Mapping {
        generated_line: 0,
        generated_column: 14,
        original: Some(OriginalLocation::new(1, 0, 0, None))
      },
    ]
  );
  let expected = serde_json::json!({
      "version": 3, "sourceRoot": null, "mappings": "ACAA,cDCA",
      "sources": ["a.css", "b.css"], "sourcesContent": [entry, imported], "names": []
  });
  assert_eq!(
    serde_json::from_str::<serde_json::Value>(&map.to_json(None).unwrap()).unwrap(),
    expected
  );
  // Reusing the actual map APIs must preserve every decoded location/content field.
  let mut decoded = SourceMap::from_json(directory.path().to_str().unwrap(), &map.to_json(None).unwrap()).unwrap();
  assert_eq!(decoded.get_mappings(), map.get_mappings());
  assert_eq!(decoded.to_json(None).unwrap(), map.to_json(None).unwrap());
}
