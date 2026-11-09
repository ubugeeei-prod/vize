use super::{Checked, Value, roster_hash, support};
use serde_json::json;
use std::fs;

const UPDATE: &str = "malformed-membership/provider-dependency-followup-12318.json";
const UPDATE_HASH: &str = "450ab3fa960006a0b624cf7195e848baf72fd8ad3961d27fba394c2b5489e911";
const PREVIOUS: &str = "malformed-membership/provider-dependency-update-12212.json";
const PREVIOUS_BINDING: &str = "tests/_fixtures/differential/typechecker/empty-vue-star-exports/malformed-membership/provider-dependency-update-12212.json";
const BEFORE: &str = "providers/@voidzero-dev+vite-plus-core@0.1.24_@types+node@25.9.2_esbuild@0.28.1_jiti@2.7.0_sass@1_648447c260f78446a04fe3ae6b71971f/node_modules/@voidzero-dev/vite-plus-core/";
const AFTER: &str = "providers/@voidzero-dev+vite-plus-core@0.1.24_@tsdown+css@0.22.14_@types+node@25.9.2_esbuild@0.28_9265ad5f7eb1fabf32cab9735a22a537/node_modules/@voidzero-dev/vite-plus-core/";

fn bound(binding: &Value, path: &str, body: &[u8]) -> bool {
    binding["path"] == path
        && binding["bytes"] == json!(body.len())
        && binding["sha256"] == support::hash(body)
}

fn load() -> Value {
    let bytes = fs::read(support::corpus().join(UPDATE)).unwrap();
    assert_eq!(support::hash(&bytes), UPDATE_HASH);
    let record: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(
        record["kind"],
        "exact-provider-roster-dependency-followup-successor"
    );
    assert_eq!(record["nativeAcceptanceCredit"], false);
    record
}

fn is_provider_path_transition(before: &str, after: &str) -> bool {
    matches!(
        (before.strip_prefix(BEFORE), after.strip_prefix(AFTER)),
        (Some(before), Some(after)) if before == after
    )
}

pub(super) fn provider_roles(previous: &Value, followup: &Value) -> Checked<Value> {
    let previous = previous["providerRoles"]
        .as_array()
        .ok_or("complete provider roles")?;
    if previous.len() != 5 {
        return Err("five complete provider roles");
    }
    let changes = followup["providerRoleManifestChanges"]
        .as_array()
        .ok_or("two provider role manifest changes")?;
    if changes.len() != 2 {
        return Err("two provider role manifest changes");
    }
    let mut current = previous.clone();
    for (entry, change) in (0..=1).zip(changes) {
        if change["entry"] != entry
            || current[entry]["manifestRelativePath"] != change["before"]
            || change["before"] == change["after"]
        {
            return Err("two exact provider role manifests");
        }
        current[entry]["manifestRelativePath"] = change["after"].clone();
    }
    Ok(Value::Array(current))
}

pub(super) fn successor(members: &[Value], previous: &Value) -> Checked<(Value, Vec<Value>)> {
    let record = load();
    let previous_bytes = fs::read(support::corpus().join(PREVIOUS))
        .map_err(|_| "complete dependency predecessor")?;
    let previous_record: Value =
        serde_json::from_slice(&previous_bytes).map_err(|_| "complete dependency predecessor")?;
    if previous != &previous_record
        || !bound(
            &record["previousProviderDependencyRecord"],
            PREVIOUS_BINDING,
            &previous_bytes,
        )
    {
        return Err("exact frozen dependency followup predecessor");
    }
    if members.len() != 86 || record["wholeBeforeRosterSha256"] != roster_hash(members) {
        return Err("complete dependency followup predecessor roster");
    }
    let changes = record["changes"]
        .as_array()
        .ok_or("six dependency followup path changes")?;
    if changes.len() != 6 {
        return Err("six dependency followup path changes");
    }
    let mut current = members.to_vec();
    for (entry, change) in (76..=81).zip(changes) {
        if change["entry"] != entry || change["before"] != members[entry] {
            return Err("six exact dependency followup entries");
        }
        let before = change["before"]
            .as_array()
            .ok_or("whole provider triplets")?;
        let after = change["after"]
            .as_array()
            .ok_or("whole provider triplets")?;
        let before_path = before[0].as_str().ok_or("literal provider paths")?;
        let after_path = after[0].as_str().ok_or("literal provider paths")?;
        if before.len() != 3
            || after.len() != 3
            || !is_provider_path_transition(before_path, after_path)
            || before[1..] != after[1..]
        {
            return Err("only the current dependency provider paths may change");
        }
        current[entry] = change["after"].clone();
    }
    if record["wholeAfterRosterSha256"] != roster_hash(&current) {
        return Err("complete dependency followup successor roster");
    }
    provider_roles(&super::load(), &record)?;
    Ok((record, current))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inputs() -> (Vec<Value>, Value) {
        let mut first: Vec<Value> =
            serde_json::from_value(support::value("malformed-membership/part-00.json")).unwrap();
        let second: Vec<Value> =
            serde_json::from_value(support::value("malformed-membership/part-01.json")).unwrap();
        first.extend(second);
        let provider = super::super::load();
        let roster = super::super::successor(&first, &provider).unwrap();
        let (dependency, roster) =
            super::super::dependency_update::successor(&roster, &provider).unwrap();
        (roster, dependency)
    }

    #[test]
    fn exact_followup_changes_only_the_six_current_provider_paths() {
        let (roster, dependency) = inputs();
        let (record, current) = successor(&roster, &dependency).unwrap();
        assert_eq!(current.len(), roster.len());
        for (index, (before, after)) in roster.iter().zip(&current).enumerate() {
            if (76..=81).contains(&index) {
                assert_eq!(after, &record["changes"][index - 76]["after"]);
            } else {
                assert_eq!(after, before);
            }
        }
    }

    #[test]
    fn dependency_path_transition_requires_both_provider_prefixes() {
        assert!(is_provider_path_transition(
            &format!("{BEFORE}package.json"),
            &format!("{AFTER}package.json"),
        ));
        assert!(!is_provider_path_transition(
            "unrelated/before/package.json",
            "unrelated/after/package.json",
        ));
    }
}
