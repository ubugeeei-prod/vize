//! Source-only whole before observations; this never rewrites a test expectation.
mod vize_bundler_before;

use lightningcss::{
  bundler::Bundler,
  css_modules::{Config, Pattern},
  error::Error,
  printer::PrinterOptions,
  stylesheet::{MinifyOptions, ParserOptions},
};
use parcel_sourcemap::SourceMap;
use serde_json::{json, Value};
use std::{fmt, fs, path::Path};
use vize_bundler_before::Provider;

fn error<T: fmt::Debug + fmt::Display>(stage: &str, value: Error<T>) -> Value {
  json!({"stage": stage, "display": value.to_string(), "kind": format!("{:?}", value.kind), "loc": value.loc})
}

fn observe(case: &Value, provider: &Provider) -> Value {
  let mut map = SourceMap::new("/");
  let options = ParserOptions {
    css_modules: case["cssModules"].as_bool().unwrap_or(false).then(|| Config {
      dashed_idents: true,
      pattern: Pattern::parse("[hash]_[local]").unwrap(),
      ..Default::default()
    }),
    ..Default::default()
  };
  let mut bundler = Bundler::new(provider, Some(&mut map), options);
  let result = match bundler.bundle(Path::new(case["entry"].as_str().unwrap())) {
    Ok(mut stylesheet) => {
      // Preserve pre-minification source/map/comment/context metadata too.
      let bundle = json!({
        "ast": stylesheet,
        "sources": stylesheet.sources,
        "sourceMapUrls": stylesheet.source_map_urls,
        "licenseComments": stylesheet.license_comments,
        "privateContentHashesObserved": false,
      });
      let minify_error = if case["minify"].as_bool().unwrap_or(false) {
        stylesheet.minify(MinifyOptions::default()).err()
      } else {
        None
      };
      if let Some(value) = minify_error {
        json!({"bundle": bundle, "error": error("minify", value)})
      } else {
        match stylesheet.to_css(PrinterOptions {
          source_map: Some(&mut map),
          ..Default::default()
        }) {
          Ok(result) => json!({
            "bundle": bundle, "code": result.code, "exports": result.exports,
            "references": result.references, "dependencies": result.dependencies,
          }),
          Err(value) => json!({"bundle": bundle, "error": error("print", value)}),
        }
      }
    }
    Err(value) => json!({"error": error("bundle", value)}),
  };
  let mappings: Vec<Value> = map
    .get_mappings()
    .iter()
    .map(|mapping| {
      json!({
        "generatedLine": mapping.generated_line,
        "generatedColumn": mapping.generated_column,
        "original": mapping.original.map(|value| json!({
          "line": value.original_line, "column": value.original_column,
          "source": value.source, "name": value.name,
        })),
      })
    })
    .collect();
  json!({
    "result": result,
    "map": {"json": map.to_json(None).unwrap(), "sources": map.get_sources(),
      "sourcesContent": map.get_sources_content(), "names": map.get_names(), "mappings": mappings},
  })
}

fn main() {
  let args: Vec<String> = std::env::args().collect();
  assert_eq!(args.len(), 5, "manifest case pool output-directory");
  let manifest: Value = serde_json::from_slice(&fs::read(&args[1]).unwrap()).unwrap();
  let case = manifest["cases"]
    .as_array()
    .unwrap()
    .iter()
    .find(|case| case["id"].as_str() == Some(args[2].as_str()))
    .unwrap();
  let threads: usize = args[3].parse().unwrap();
  assert!([1, 8, 32].contains(&threads));
  let directory = Path::new(&args[4]);
  fs::create_dir_all(directory).unwrap();
  fs::write(
    directory.join("startup.json"),
    serde_json::to_vec_pretty(&json!({
      "case": case, "threads": threads, "pid": std::process::id(),
      "sourceRevision": std::env::var("CSS_ENGINE_SOURCE_REVISION").unwrap(),
      "expectationsRecaptured": false, "publicationQualification": false,
    }))
    .unwrap(),
  )
  .unwrap();
  let provider = Provider::new(case, threads, directory.join("events.jsonl"));
  let pool = rayon::ThreadPoolBuilder::new().num_threads(threads).build().unwrap();
  let observed = pool.install(|| observe(case, &provider));
  fs::write(
    directory.join("observation.json"),
    serde_json::to_vec_pretty(&observed).unwrap(),
  )
  .unwrap();
  println!("{}", serde_json::to_string(&observed).unwrap());
}
