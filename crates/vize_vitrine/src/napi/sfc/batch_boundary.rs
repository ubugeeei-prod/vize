//! Materialize batch dependency arrays with one JavaScript string per path.
//!
//! The result arrays remain distinct and mutable. Their elements are primitive
//! strings, which JavaScript cannot mutate, so equal paths may safely share a
//! string handle. Handles and their Rust lookup keys live only for this call
//! on its JavaScript thread; no environment-bound value enters a worker or a
//! process-wide cache.

use napi::{
    Env, JsString, Property,
    bindgen_prelude::{Array, Error, JsObjectValue, Object, Result, Status, ToNapiValue},
};
use vize_l0::FxHashMap;

use super::batch_targets::BatchCompileTargetsResultNapi;
use super::types::BatchCompileResultWithFilesNapi;

fn required<T>(value: Option<T>, message: &'static str) -> Result<T> {
    value.ok_or_else(|| Error::new(Status::GenericFailure, message))
}

/// Keep the derive-generated field conversion for every non-dependency field.
/// The temporary holder is not returned; it only invokes that conversion through
/// napi's safe object API, without calling `ToNapiValue` through a raw handle.
fn object<'env, T: ToNapiValue>(env: &'env Env, packet: T) -> Result<Object<'env>> {
    let mut holder = Object::new(env)?;
    let property = Property::new()
        .with_utf8_name("packet")?
        .with_napi_value(env, packet)?;
    holder.define_properties(&[property])?;
    required(holder.get("packet")?, "compiled batch packet is missing")
}

struct DependencyStrings<'env> {
    env: &'env Env,
    values: FxHashMap<String, JsString<'env>>,
}

impl<'env> DependencyStrings<'env> {
    fn new(env: &'env Env) -> Self {
        Self {
            env,
            values: FxHashMap::default(),
        }
    }

    fn array(&mut self, paths: Vec<String>) -> Result<Array<'env>> {
        let length = u32::try_from(paths.len())
            .map_err(|_| Error::new(Status::InvalidArg, "too many dependency paths"))?;
        let env = self.env;
        let mut array = env.create_array(length)?;
        for (index, path) in paths.into_iter().enumerate() {
            let value = match self.values.get(path.as_str()) {
                Some(value) => *value,
                None => {
                    let value = env.create_string(&path)?;
                    self.values.insert(path, value);
                    value
                }
            };
            array.set(index as u32, &value)?;
        }
        Ok(array)
    }

    fn restore(&mut self, rows: Array<'env>, paths: Vec<Option<Vec<String>>>) -> Result<()> {
        for (index, paths) in paths.into_iter().enumerate() {
            if let Some(paths) = paths {
                let mut row: Object<'env> =
                    required(rows.get(index as u32)?, "compiled batch row is missing")?;
                row.set("dependencies", self.array(paths)?)?;
            }
        }
        Ok(())
    }
}

pub(super) fn with_results<'env>(
    env: &'env Env,
    mut packet: BatchCompileResultWithFilesNapi,
) -> Result<Object<'env>> {
    if packet.results.iter().all(|row| row.dependencies.is_none()) {
        return object(env, packet);
    }
    let paths = packet
        .results
        .iter_mut()
        .map(|row| row.dependencies.take())
        .collect();
    let packet = object(env, packet)?;
    let rows = required(packet.get("results")?, "compiled batch results are missing")?;
    DependencyStrings::new(env).restore(rows, paths)?;
    Ok(packet)
}

pub(super) fn for_targets<'env>(
    env: &'env Env,
    mut packet: BatchCompileTargetsResultNapi,
) -> Result<Object<'env>> {
    if packet
        .targets
        .iter()
        .flat_map(|target| &target.results)
        .all(|row| row.dependencies.is_none())
    {
        return object(env, packet);
    }
    let paths: Vec<Vec<_>> = packet
        .targets
        .iter_mut()
        .map(|target| {
            target
                .results
                .iter_mut()
                .map(|row| row.dependencies.take())
                .collect()
        })
        .collect();
    let packet = object(env, packet)?;
    let targets: Array<'env> =
        required(packet.get("targets")?, "compiled batch targets are missing")?;
    let mut strings = DependencyStrings::new(env);
    for (index, paths) in paths.into_iter().enumerate() {
        let target: Object<'env> =
            required(targets.get(index as u32)?, "compiled target is missing")?;
        let rows = required(
            target.get("results")?,
            "compiled target results are missing",
        )?;
        strings.restore(rows, paths)?;
    }
    Ok(packet)
}
