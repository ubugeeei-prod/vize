//! Whole source-order controls, retaining the immutable observed before inputs.
use super::*;
use crate::{css_modules, printer::PrinterOptions, stylesheet::MinifyOptions};
use serde_json::Value;
use std::{collections::HashMap, io, sync::Condvar, time::Duration};

struct Provider {
  files: HashMap<PathBuf, String>,
  aliases: HashMap<String, PathBuf>,
  entry: PathBuf,
  first: Option<String>,
  target: Option<PathBuf>,
  threads: usize,
  read: Mutex<bool>,
  wake: Condvar,
}

impl SourceProvider for Provider {
  // io::Error deliberately has no Clone requirement.
  type Error = io::Error;
  fn read<'a>(&'a self, file: &Path) -> Result<&'a str, Self::Error> {
    if self.target.as_deref() == Some(file) {
      *self.read.lock().unwrap() = true;
      self.wake.notify_all();
    }
    self
      .files
      .get(file)
      .map(String::as_str)
      .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, format!("fixture missing {}", file.display())))
  }
  fn resolve(&self, specifier: &str, origin: &Path) -> Result<ResolveResult, Self::Error> {
    if self.threads > 1 && origin == self.entry && self.first.as_deref() == Some(specifier) {
      let (read, _) = self
        .wake
        .wait_timeout_while(self.read.lock().unwrap(), Duration::from_secs(3), |read| !*read)
        .unwrap();
      assert!(*read, "the later physical read must precede the first resolve");
    }
    if specifier.starts_with("https:") {
      Ok(ResolveResult::External(specifier.to_owned()))
    } else {
      Ok(
        self
          .aliases
          .get(specifier)
          .cloned()
          .unwrap_or_else(|| origin.with_file_name(specifier))
          .into(),
      )
    }
  }
}

fn provider(case: &Value, threads: usize) -> Provider {
  Provider {
    files: case["files"]
      .as_object()
      .unwrap()
      .iter()
      .map(|(name, source)| (PathBuf::from(name), source.as_str().unwrap().to_owned()))
      .collect(),
    aliases: case["aliases"]
      .as_object()
      .unwrap()
      .iter()
      .map(|(name, path)| (name.clone(), PathBuf::from(path.as_str().unwrap())))
      .collect(),
    entry: PathBuf::from(case["entry"].as_str().unwrap()),
    first: case["forceReadBeforeResolve"]["specifier"].as_str().map(str::to_owned),
    target: case["forceReadBeforeResolve"]["path"].as_str().map(PathBuf::from),
    threads,
    read: Mutex::new(false),
    wake: Condvar::new(),
  }
}

#[test]
fn whole_original_and_forced_authored_contexts() {
  let inputs: Value =
    serde_json::from_str(include_str!("../../tests/vize_fixtures/bundler-before-inputs.json")).unwrap();
  assert_eq!(inputs["cases"].as_array().unwrap().len(), 21);
  for threads in [1, 8, 32] {
    let pool = rayon::ThreadPoolBuilder::new().num_threads(threads).build().unwrap();
    for case in inputs["cases"].as_array().unwrap() {
      pool.install(|| {
        let fs = provider(case, threads);
        let modules = case["cssModules"].as_bool().unwrap_or(false);
        let mut map = SourceMap::new("/");
        let mut bundler = Bundler::new(&fs, Some(&mut map), ParserOptions {
          css_modules: modules.then(|| css_modules::Config {
            dashed_idents: true, pattern: css_modules::Pattern::parse("[hash]_[local]").unwrap(), ..Default::default()
          }), ..Default::default()
        });
        let id = case["id"].as_str().unwrap();
        let result = bundler.bundle(&fs.entry);
        if let Err(error) = result {
          let (kind, filename, line, column) = match id {
            "competing-io-layer-error" => ("UnsupportedLayerCombination", "/a.css", 1, 1),
            "competing-parser-condition-error" => ("UnsupportedImportCondition", "/a.css", 1, 1),
            "external-after-bundled-constraint" => ("ExternalImportAfterBundledImport", "", 1, 1),
            "css-module-competing-external-error" => ("ResolverError(Custom { kind: NotFound, error: \"fixture missing /missing.css\" })", "/a.css", 0, 1),
            _ => panic!("unexpected {id}/pool{threads}: {error:?}"),
          };
          assert_eq!(format!("{:?}", error.kind), kind);
          assert_eq!(error.loc, Some(ErrorLocation { filename: filename.to_owned(), line, column }));
          let display = match id {
            "competing-io-layer-error" => "Unsupported layer combination in @import at /a.css:1:1",
            "competing-parser-condition-error" => "Unsupported import condition at /a.css:1:1",
            "external-after-bundled-constraint" => "An external `@import` was found after a bundled `@import`. This may result in unintended selector order. at :1:1",
            "css-module-competing-external-error" => "fixture missing /missing.css at /a.css:0:1",
            _ => unreachable!(),
          };
          assert_eq!(error.to_string(), display);
          return;
        }
        let mut stylesheet = result.unwrap();
        if case["minify"].as_bool().unwrap_or(false) {
          stylesheet.minify(MinifyOptions::default()).unwrap();
        }
        let printed = stylesheet.to_css(PrinterOptions { source_map: Some(&mut map), ..Default::default() }).unwrap();
        assert!(!matches!(id, "competing-io-layer-error" | "competing-parser-condition-error" | "external-after-bundled-constraint" | "css-module-competing-external-error"), "expected a complete error for {id}/pool{threads}");
        let delayed_expected = if id == "delayed-multifile-map" {
          // The read gate does not order sibling parse completion. Preserve
          // the complete actual license vector; require each original member
          // exactly once without inventing a stable cross-pool prefix order.
          let members = ["! entry license ", "! b license ", "! c license ", "! d license "];
          assert_eq!(stylesheet.license_comments.len(), members.len());
          for member in members {
            assert_eq!(stylesheet.license_comments.iter().filter(|value| value.as_ref() == member).count(), 1);
          }
          let mut expected = String::new();
          for member in &stylesheet.license_comments {
            expected.push_str(&format!("/*{member}*/\n"));
          }
          expected.push_str(".d {\n  color: #000;\n}\n\n.b {\n  color: green;\n}\n\n.c {\n  color: #00f;\n}\n\n.a {\n  color: red;\n}\n");
          Some(expected)
        } else {
          None
        };
        let expected = delayed_expected.as_deref().or_else(|| case["wholeExpected"].as_str()).unwrap_or_else(|| match id {
          "first-context-diamond" => "@media print, screen {\n  .b {\n    color: #ff0;\n  }\n}\n\n@media print and (color), screen and (color) {\n  .c {\n    color: green;\n  }\n}\n\n@media screen {\n  .d {\n    color: #00f;\n  }\n}\n\n.a {\n  color: red;\n}\n",
          "nested-layer-statements" => "@layer foo.one, foo.two;\n@layer foo.bar.three;\n\n@layer foo.bar {\n  .c {\n    color: #00f;\n  }\n}\n\n@layer foo {\n  .b {\n    color: green;\n  }\n}\n",
          "css-module-mixed-import-composes" => "._9z6RGq_x {\n  color: green;\n}\n\n._6lixEq_a {\n  color: red;\n}\n",
          "ordinary-self-import" => ".a {\n  color: red;\n}\n",
          "ordinary-two-file-cycle" => ".b {\n  color: green;\n}\n\n.a {\n  color: red;\n}\n",
          "css-module-first-root-self-composes" => "._6lixEq_a {\n  color: red;\n}\n\n._6lixEq_b {\n  color: green;\n}\n",
          _ => panic!("no explicit source-order reference for {id}"),
        });
        assert_eq!(printed.code, expected, "{id}/pool{threads}");
        if let Some(exports) = printed.exports {
          assert!(modules);
          let self_composes = id == "css-module-first-root-self-composes";
          assert_eq!(exports.len(), if self_composes { 2 } else { 1 });
          let a = exports.get("a").unwrap();
          assert_eq!(a.name, "_6lixEq_a");
          if self_composes {
            // Removing the recursive emission edge retains the printer's
            // existing forward SourceIndex lookup; it does not invent local
            // composition equivalence for a target declared later.
            assert!(a.composes.is_empty());
          } else {
            assert_eq!(a.composes, vec![css_modules::CssModuleReference::Local {
              name: "_9z6RGq_x".to_owned(),
            }]);
          }
          assert!(!a.is_referenced);
          if self_composes {
            let b = exports.get("b").unwrap();
            assert_eq!(b.name, "_6lixEq_b");
            assert!(b.composes.is_empty());
            assert!(!b.is_referenced);
          }
          assert!(printed.references.as_ref().is_some_and(|references| references.is_empty()));
        } else {
          assert!(!modules);
          assert!(printed.references.is_none());
        }
        assert!(printed.dependencies.is_none());
        // Keep every physical map field; this does not sort/rewrite handles or
        // claim a single cross-scheduler map/reference for sibling discovery.
        let json = map.to_json(None).unwrap();
        let mut decoded = SourceMap::from_json("/", &json).unwrap();
        assert_eq!(decoded.to_json(None).unwrap(), json);
        assert_eq!(decoded.get_mappings(), map.get_mappings());
        assert_eq!(decoded.get_sources(), map.get_sources());
        assert_eq!(decoded.get_sources_content(), map.get_sources_content());
        assert_eq!(decoded.get_names(), map.get_names());
      });
    }
  }
}

#[test]
fn same_owner_backward_composes_preserves_transitive_exports() {
  // A healthy non-root backward reference must keep SourceIndex semantics;
  // changing it to local syntax drops the transitive c export from root.a.
  for threads in [1, 8, 32] {
    let pool = rayon::ThreadPoolBuilder::new().num_threads(threads).build().unwrap();
    pool.install(|| {
      let case = serde_json::json!({
        "entry": "/a.css", "aliases": {}, "files": {
          "/a.css": "@import \"b.css\";.root{composes:a from \"b.css\";color:black}",
          "/b.css": ".c{color:red}.b{composes:c;color:blue}.a{composes:b from \"b.css\";color:green}"
        }
      });
      let fs = provider(&case, threads);
      let mut bundler = Bundler::new(&fs, None, ParserOptions {
        css_modules: Some(css_modules::Config {
          pattern: css_modules::Pattern::parse("[hash]_[local]").unwrap(), ..Default::default()
        }), ..Default::default()
      });
      let mut sheet = bundler.bundle(&fs.entry).unwrap();
      sheet.minify(MinifyOptions::default()).unwrap();
      let printed = sheet.to_css(PrinterOptions::default()).unwrap();
      assert_eq!(printed.code, "._9z6RGq_c {\n  color: red;\n}\n\n._9z6RGq_b {\n  color: #00f;\n}\n\n._9z6RGq_a {\n  color: green;\n}\n\n._6lixEq_root {\n  color: #000;\n}\n");
      let exports = printed.exports.unwrap();
      assert_eq!(exports.len(), 1);
      let root = exports.get("root").unwrap();
      assert_eq!(root.name, "_6lixEq_root");
      assert!(!root.is_referenced);
      assert_eq!(root.composes, ["a", "b", "c"].map(|name| css_modules::CssModuleReference::Local {
        name: format!("_9z6RGq_{name}"),
      }));
      assert!(printed.references.unwrap().is_empty());
      assert!(printed.dependencies.is_none());
    });
  }
}

#[test]
fn same_owner_dashed_reference_keeps_export_usage_flags() {
  for threads in [1, 8, 32] {
    let pool = rayon::ThreadPoolBuilder::new().num_threads(threads).build().unwrap();
    pool.install(|| {
      let case = serde_json::json!({
        "entry": "/a.css", "aliases": {}, "files": {
          "/a.css": ":root{--tone:red}.a{color:var(--tone from \"a.css\")}" }
      });
      let fs = provider(&case, threads);
      let mut bundler = Bundler::new(
        &fs,
        None,
        ParserOptions {
          css_modules: Some(css_modules::Config {
            dashed_idents: true,
            pattern: css_modules::Pattern::parse("[hash]_[local]").unwrap(),
            ..Default::default()
          }),
          ..Default::default()
        },
      );
      let mut sheet = bundler.bundle(&fs.entry).unwrap();
      sheet.minify(MinifyOptions::default()).unwrap();
      let printed = sheet.to_css(PrinterOptions::default()).unwrap();
      assert_eq!(
        printed.code,
        ":root {\n  --_6lixEq_tone: red;\n}\n\n._6lixEq_a {\n  color: var(--_6lixEq_tone);\n}\n"
      );
      let exports = printed.exports.unwrap();
      assert_eq!(exports.len(), 2);
      for (key, name) in [("a", "_6lixEq_a"), ("--tone", "--_6lixEq_tone")] {
        let export = exports.get(key).unwrap();
        assert_eq!(export.name, name);
        assert!(export.composes.is_empty());
        assert!(!export.is_referenced);
      }
      assert!(printed.references.unwrap().is_empty());
      assert!(printed.dependencies.is_none());
    });
  }
}
