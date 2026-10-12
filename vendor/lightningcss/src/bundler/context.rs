//! Replay import conditions in the existing logical ordering phase.
use super::*;

fn filename(indexes: &DashMap<PathBuf, u32>, source: u32) -> String {
  indexes
    .iter()
    .find(|entry| *entry.value() == source)
    .unwrap()
    .key()
    .to_str()
    .unwrap()
    .to_owned()
}

fn location(indexes: &DashMap<PathBuf, u32>, loc: Location) -> ErrorLocation {
  ErrorLocation::new(loc, filename(indexes, loc.source_index))
}

fn fold<'a, T, E: std::error::Error>(
  entry: &mut BundleStyleSheet<'a, T>,
  rule: ImportRule<'a>,
  indexes: &DashMap<PathBuf, u32>,
) -> Result<(), Error<BundleErrorKind<'a, E>>> {
  if (!rule.media.media_queries.is_empty() && entry.supports.is_some())
    || (!entry.media.media_queries.is_empty() && rule.supports.is_some())
  {
    return Err(Error {
      kind: BundleErrorKind::UnsupportedImportCondition,
      loc: Some(location(indexes, rule.loc)),
    });
  }
  if rule.media.media_queries.is_empty() {
    entry.media.media_queries.clear();
  } else if !entry.media.media_queries.is_empty() {
    entry.media.or(&rule.media);
  }
  if let Some(supports) = rule.supports {
    if let Some(existing) = &mut entry.supports {
      existing.or(&supports);
    }
  } else {
    entry.supports = None;
  }
  if let Some(layer) = &rule.layer {
    if let Some(existing) = &entry.layer {
      if layer != existing || (layer.is_none() && existing.is_none()) {
        return Err(Error {
          kind: BundleErrorKind::UnsupportedLayerCombination,
          loc: Some(location(indexes, rule.loc)),
        });
      }
    } else {
      entry.layer = rule.layer;
    }
  }
  Ok(())
}

fn combine<'a, E: std::error::Error>(
  parent: &ImportRule<'a>,
  import: &ImportRule<'a>,
  indexes: &DashMap<PathBuf, u32>,
) -> Result<ImportRule<'a>, Error<BundleErrorKind<'a, E>>> {
  let mut media = parent.media.clone();
  media.and(&import.media).map_err(|_| Error {
    kind: BundleErrorKind::UnsupportedMediaBooleanLogic,
    loc: Some(location(indexes, import.loc)),
  })?;
  let layer = if (parent.layer == Some(None) && import.layer.is_some())
    || (import.layer == Some(None) && parent.layer.is_some())
  {
    return Err(Error {
      kind: BundleErrorKind::UnsupportedLayerCombination,
      loc: Some(location(indexes, import.loc)),
    });
  } else if let Some(Some(a)) = &parent.layer {
    if let Some(Some(b)) = &import.layer {
      let mut name = a.clone();
      name.0.extend(b.0.iter().cloned());
      Some(Some(name))
    } else {
      Some(Some(a.clone()))
    }
  } else {
    import.layer.clone()
  };
  Ok(ImportRule {
    layer,
    media,
    supports: combine_supports(parent.supports.clone(), &import.supports),
    url: "".into(),
    loc: import.loc,
  })
}

fn remove_self_emission<'a, T>(entry: &mut BundleStyleSheet<'a, T>, source_index: u32) {
  // A same-owner SourceIndex retains its existing printer semantics, including
  // backward transitive composes and dashed-reference export flags. Only its
  // redundant emission edge is removed.
  // Keeping the root self edge would match the old default parent0/dep0 and
  // recurse forever even after its rule vector has been consumed.
  entry
    .css_modules_deps
    .retain(|dependency| !matches!(dependency.source, Dependency::File(index) if index == source_index));
}

pub(super) fn replay<'a, T, E: std::error::Error>(
  stylesheets: &mut Vec<BundleStyleSheet<'a, T>>,
  errors: &mut Vec<Option<Error<BundleErrorKind<'a, E>>>>,
  indexes: &DashMap<PathBuf, u32>,
  visited: &mut HashSet<u32>,
) -> Result<(), Error<BundleErrorKind<'a, E>>> {
  fn process<'a, T, E: std::error::Error>(
    stylesheets: &mut Vec<BundleStyleSheet<'a, T>>,
    errors: &mut Vec<Option<Error<BundleErrorKind<'a, E>>>>,
    indexes: &DashMap<PathBuf, u32>,
    source: u32,
    rule: ImportRule<'a>,
    visited: &mut HashSet<u32>,
  ) -> Result<(), Error<BundleErrorKind<'a, E>>> {
    let entry = &mut stylesheets[source as usize];
    if !visited.insert(source) {
      return fold(entry, rule, indexes);
    }
    if let Some(index) = entry.load_error {
      let mut error = errors[index].take().unwrap();
      if matches!(error.kind, BundleErrorKind::ResolverError(_)) {
        error.loc = (rule.loc.column != 0).then(|| location(indexes, rule.loc));
      }
      return Err(error);
    }
    entry.layer = rule.layer.clone();
    entry.media = rule.media.clone();
    entry.supports = rule.supports.clone();
    entry.loc = rule.loc;
    let count = entry.stylesheet.as_ref().unwrap().rules.0.len();
    let mut import_index = 0;
    for ordinal in 0..count {
      let item = &mut stylesheets[source as usize].stylesheet.as_mut().unwrap().rules.0[ordinal];
      if let CssRule::LayerStatement(layer) = item {
        if let Some(Some(parent)) = &rule.layer {
          for name in &mut layer.names {
            name.0.insert_many(0, parent.0.iter().cloned());
          }
        }
      }
      let import = match item {
        CssRule::Import(import) => import.clone(),
        _ => continue,
      };
      let incoming = combine(&rule, &import, indexes)?;
      let dependency = stylesheets[source as usize].dependencies[import_index].clone();
      import_index += 1;
      match dependency {
        Dependency::File(child) => process(stylesheets, errors, indexes, child, incoming, visited)?,
        Dependency::Failed(index) => return Err(errors[index].take().unwrap()),
        Dependency::External(_) => {}
      }
    }
    assert_eq!(import_index, stylesheets[source as usize].dependencies.len());
    for ordinal in 0..stylesheets[source as usize].css_modules_deps.len() {
      let dependency = stylesheets[source as usize].css_modules_deps[ordinal].clone();
      let mut incoming = rule.clone();
      incoming.loc = dependency.loc;
      match dependency.source {
        Dependency::File(child) => process(stylesheets, errors, indexes, child, incoming, visited)?,
        Dependency::Failed(index) => return Err(errors[index].take().unwrap()),
        Dependency::External(_) => unreachable!("module external errors were retained"),
      }
    }
    // Validate repeated contexts before pruning only the emission edge.
    remove_self_emission(&mut stylesheets[source as usize], source);
    Ok(())
  }
  process(
    stylesheets,
    errors,
    indexes,
    0,
    ImportRule {
      url: "".into(),
      layer: None,
      supports: None,
      media: MediaList::new(),
      loc: Location {
        source_index: 0,
        line: 0,
        column: 0,
      },
    },
    visited,
  )
}
