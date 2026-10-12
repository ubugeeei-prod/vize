//! Archive the full stable authority once; validate retained retirements without network.
use super::{Source, Stable, body_with, github, marker, plan, read_plan, supports_cut};
use serde_json::{Value, json};
use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

pub(in super::super) fn capture(
    source: &Source,
    main: &str,
    root: &Path,
) -> Result<Option<Value>, String> {
    capture_with(source, main, root, &|route| {
        github::api(&source.candidate.repository, route, root)
    })
}

pub(super) fn capture_with(
    source: &Source,
    main: &str,
    root: &Path,
    api: &dyn Fn(&str) -> Result<Value, String>,
) -> Result<Option<Value>, String> {
    let Some(context) = &source.stable else {
        return Ok(None);
    };
    let (actual, value) = read_plan(&context.policy, root)?;
    if actual.blob != context.blob || actual.sha256 != context.sha256 {
        return Err("Stable archive authority changed".into());
    }
    let responses = RefCell::new(BTreeMap::<String, Value>::new());
    plan::verify_against(
        &value,
        context,
        &source.cut,
        &source.candidate.repository,
        main,
        root,
        &|route| {
            let value = api(route)?;
            if responses
                .borrow_mut()
                .insert(route.into(), value.clone())
                .is_some()
            {
                return Err("Duplicate stable archive response".into());
            }
            Ok(value)
        },
    )?;
    Ok(Some(
        json!({"policy":context.policy,"blob":context.blob,"sha256":context.sha256,"responses":responses.into_inner()}),
    ))
}

pub(in super::super) fn from_archive(
    body: &str,
    cut: &str,
    tag: &str,
    repository: &str,
    main: &str,
    evidence: Option<&Value>,
    root: &Path,
) -> Result<Option<Stable>, String> {
    let context = body_with(body, &|policy| {
        let evidence = evidence.ok_or("Missing complete archived stable authority")?;
        let keys: BTreeSet<_> = evidence
            .as_object()
            .ok_or("Missing stable evidence object")?
            .keys()
            .map(String::as_str)
            .collect();
        if keys
            != ["policy", "blob", "sha256", "responses"]
                .into_iter()
                .collect()
        {
            return Err("Unknown or omitted archived stable evidence".into());
        }
        let (context, value) = read_plan(policy, root)?;
        if evidence["policy"] != policy
            || evidence["blob"] != context.blob
            || evidence["sha256"] != context.sha256
        {
            return Err("Archived stable policy/blob/digest mismatch".into());
        }
        let responses = evidence["responses"]
            .as_object()
            .ok_or("Missing full stable API snapshots")?;
        let used = RefCell::new(BTreeSet::new());
        plan::verify_against(&value, &context, cut, repository, main, root, &|route| {
            used.borrow_mut().insert(route.to_owned());
            responses
                .get(route)
                .cloned()
                .ok_or(format!("Missing archived stable API response {route}"))
        })?;
        if *used.borrow() != responses.keys().cloned().collect() {
            return Err("Unknown or unconsumed archived stable API snapshots".into());
        }
        supports_cut(&context, cut, root)?;
        if marker(body, "vize-release-bump")? != "patch" || value["target"] != tag {
            return Err("Archived stable receipt target/bump differs".into());
        }
        Ok(context)
    })?;
    if context.is_none() && evidence.is_some() {
        return Err("A normal source cannot inherit archived stable authority".into());
    }
    Ok(context)
}
