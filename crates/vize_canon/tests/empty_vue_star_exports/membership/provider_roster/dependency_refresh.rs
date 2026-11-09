use super::{Checked, Value, dependency_update, roster_hash, support};
use serde_json::json;
use std::fs;

const UPDATE: &str = "malformed-membership/provider-dependency-refresh-update-12212.json";
const UPDATE_HASH: &str = "2c2ac89d1c11a17e060a95c0837f0bdb1b04381901021aa3227d95c885355eaf";
const PREVIOUS_BINDING: &str = "tests/_fixtures/differential/typechecker/empty-vue-star-exports/malformed-membership/provider-dependency-update-12212.json";
const QUALIFICATION: &str = "malformed-membership/provider-js-tooling-refresh-update-12212.json";
const QUALIFICATION_BINDING: &str = "tests/_fixtures/differential/typechecker/empty-vue-star-exports/malformed-membership/provider-js-tooling-refresh-update-12212.json";

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
        "exact-provider-roster-dependency-refresh-successor"
    );
    assert_eq!(record["nativeAcceptanceCredit"], false);
    record
}

pub(super) fn successor(
    members: &[Value],
    previous: &Value,
) -> Checked<(Value, Vec<Value>, Value)> {
    let record = load();
    let previous_bytes = fs::read(support::corpus().join(dependency_update::UPDATE))
        .map_err(|_| "complete dependency provider predecessor")?;
    let qualification_bytes = fs::read(support::corpus().join(QUALIFICATION))
        .map_err(|_| "complete JavaScript tooling qualification record")?;
    let previous_record: Value = serde_json::from_slice(&previous_bytes)
        .map_err(|_| "complete dependency provider predecessor")?;
    let qualification_record: Value = serde_json::from_slice(&qualification_bytes)
        .map_err(|_| "complete JavaScript tooling qualification record")?;
    if previous != &previous_record
        || qualification_record["version"] != 1
        || qualification_record["kind"]
            != "exact-provider-qualification-js-tooling-refresh-successor"
        || qualification_record["nativeAcceptanceCredit"] != false
        || !bound(
            &record["previousProviderDependencyRecord"],
            PREVIOUS_BINDING,
            &previous_bytes,
        )
        || !bound(
            &record["providerQualificationRecord"],
            QUALIFICATION_BINDING,
            &qualification_bytes,
        )
    {
        return Err("exact frozen dependency refresh predecessor");
    }
    if members.len() != 86 || record["wholeBeforeRosterSha256"] != roster_hash(members) {
        return Err("complete dependency refresh predecessor roster");
    }
    let changes = record["changes"]
        .as_array()
        .ok_or("six dependency refresh path changes")?;
    let entries = [76, 77, 78, 79, 80, 81];
    if changes.len() != entries.len() {
        return Err("six dependency refresh path changes");
    }
    const BEFORE: &str = "providers/@voidzero-dev+vite-plus-core@0.1.24_@types+node@25.9.2_esbuild@0.28.1_jiti@2.7.0_sass@1_648447c260f78446a04fe3ae6b71971f/node_modules/@voidzero-dev/vite-plus-core/";
    const AFTER: &str = "providers/@voidzero-dev+vite-plus-core@0.1.24_@tsdown+css@0.22.14_@types+node@25.9.2_esbuild@0.28_9265ad5f7eb1fabf32cab9735a22a537/node_modules/@voidzero-dev/vite-plus-core/";
    let mut current = members.to_vec();
    for (&entry, change) in entries.iter().zip(changes) {
        if change["entry"] != entry || change["before"] != members[entry] {
            return Err("six exact dependency refresh path changes");
        }
        let before = change["before"]
            .as_array()
            .ok_or("whole provider triplets")?;
        let after = change["after"]
            .as_array()
            .ok_or("whole provider triplets")?;
        if before.len() != 3 || after.len() != 3 {
            return Err("whole provider triplets");
        }
        let before_path = before[0].as_str().ok_or("literal provider paths")?;
        let after_path = after[0].as_str().ok_or("literal provider paths")?;
        if before_path.strip_prefix(BEFORE) != after_path.strip_prefix(AFTER)
            || !before_path.starts_with(BEFORE)
            || before[1..] != after[1..]
        {
            return Err("only the six declared dependency refresh paths may change");
        }
        current[entry] = change["after"].clone();
    }
    if record["wholeAfterRosterSha256"] != roster_hash(&current) {
        return Err("complete dependency refresh successor roster");
    }
    let mut provider_record = super::load();
    let roles = provider_record["providerRoles"]
        .as_array_mut()
        .ok_or("complete provider roles")?;
    if record["wholeBeforeProviderRolesSha256"] != roster_hash(roles) {
        return Err("complete dependency refresh predecessor roles");
    }
    let role_changes = record["providerRoleChanges"]
        .as_array()
        .ok_or("two dependency refresh role changes")?;
    if role_changes.len() != 2 {
        return Err("two dependency refresh role changes");
    }
    for (&entry, change) in [0, 1].iter().zip(role_changes) {
        if change["entry"] != entry
            || roles[entry]["manifestRelativePath"] != change["beforeManifestRelativePath"]
        {
            return Err("two exact dependency refresh role changes");
        }
        roles[entry]["manifestRelativePath"] = change["afterManifestRelativePath"].clone();
    }
    if record["wholeAfterProviderRolesSha256"] != roster_hash(roles) {
        return Err("complete dependency refresh successor roles");
    }
    Ok((record, current, provider_record))
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
        let previous_roster = super::super::load();
        let roster = super::super::successor(&first, &previous_roster).unwrap();
        let previous = dependency_update::load();
        let (_, roster) = dependency_update::successor(&roster, &previous_roster).unwrap();
        (roster, previous)
    }

    #[test]
    fn exact_refresh_changes_only_the_six_provider_paths() {
        let (roster, previous) = inputs();
        let (record, current, provider_record) = successor(&roster, &previous).unwrap();
        for (index, (before, after)) in roster.iter().zip(&current).enumerate() {
            if let Some(change) = [76, 77, 78, 79, 80, 81]
                .iter()
                .position(|entry| *entry == index)
            {
                assert_eq!(after, &record["changes"][change]["after"]);
            } else {
                assert_eq!(after, before);
            }
        }
        assert_eq!(
            provider_record["providerRoles"][0]["manifestRelativePath"],
            record["providerRoleChanges"][0]["afterManifestRelativePath"]
        );
        assert_eq!(
            provider_record["providerRoles"][1]["manifestRelativePath"],
            record["providerRoleChanges"][1]["afterManifestRelativePath"]
        );
    }

    #[test]
    fn foreign_predecessors_bodies_and_paths_are_refused() {
        let (roster, previous) = inputs();
        let mut foreign_previous = previous.clone();
        foreign_previous["wholeAfterRosterSha256"] = json!(support::hash(b"foreign"));
        assert_eq!(
            successor(&roster, &foreign_previous),
            Err("exact frozen dependency refresh predecessor")
        );
        let mut body = roster.clone();
        body[76][2] = json!(support::hash(b"foreign"));
        assert_eq!(
            successor(&body, &previous),
            Err("complete dependency refresh predecessor roster")
        );
    }
}
