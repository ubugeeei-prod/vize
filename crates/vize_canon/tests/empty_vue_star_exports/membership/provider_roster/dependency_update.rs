use super::{Checked, Value, roster_hash, support};
use serde_json::json;
use std::fs;

const UPDATE: &str = "malformed-membership/provider-dependency-update-12212.json";
const UPDATE_HASH: &str = "94361b7ab53073c77800589b5ff1bf00b7d65b85826dee8fbe3f263e53336c78";
const PREVIOUS: &str = "malformed-membership/provider-peer-update-20261031.json";
const PREVIOUS_BINDING: &str = "tests/_fixtures/differential/typechecker/empty-vue-star-exports/malformed-membership/provider-peer-update-20261031.json";
const QUALIFICATION: &str =
    "malformed-membership/provider-root-dependency-ownership-update-12123.json";
const QUALIFICATION_BINDING: &str = "tests/_fixtures/differential/typechecker/empty-vue-star-exports/malformed-membership/provider-root-dependency-ownership-update-12123.json";

fn bound(binding: &Value, path: &str, body: &[u8]) -> bool {
    binding["path"] == path
        && binding["bytes"] == json!(body.len())
        && binding["sha256"] == support::hash(body)
}

fn load() -> Value {
    let bytes = fs::read(support::corpus().join(UPDATE)).unwrap();
    assert_eq!(support::hash(&bytes), UPDATE_HASH);
    let record: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(record["version"], 1);
    assert_eq!(record["kind"], "exact-provider-roster-dependency-successor");
    assert_eq!(record["nativeAcceptanceCredit"], false);
    record
}

pub(super) fn successor(
    members: &[Value],
    previous: &Value,
    qualification: &Value,
) -> Checked<(Value, Vec<Value>)> {
    let record = load();
    let previous_bytes = fs::read(support::corpus().join(PREVIOUS))
        .map_err(|_| "complete provider roster predecessor")?;
    let qualification_bytes = fs::read(support::corpus().join(QUALIFICATION))
        .map_err(|_| "complete provider qualification record")?;
    let previous_record: Value = serde_json::from_slice(&previous_bytes)
        .map_err(|_| "complete provider roster predecessor")?;
    let qualification_record: Value = serde_json::from_slice(&qualification_bytes)
        .map_err(|_| "complete provider qualification record")?;
    if previous != &previous_record
        || qualification != &qualification_record
        || !bound(
            &record["previousProviderRosterRecord"],
            PREVIOUS_BINDING,
            &previous_bytes,
        )
        || !bound(
            &record["providerQualificationRecord"],
            QUALIFICATION_BINDING,
            &qualification_bytes,
        )
    {
        return Err("exact frozen dependency provider predecessor");
    }
    if members.len() != 86 || record["wholeBeforeRosterSha256"] != roster_hash(members) {
        return Err("complete dependency provider predecessor roster");
    }
    let changes = record["changes"]
        .as_array()
        .ok_or("eight dependency provider path changes")?;
    let entries = [64, 65, 76, 77, 78, 79, 80, 81];
    if changes.len() != entries.len() {
        return Err("eight dependency provider path changes");
    }
    let mut current = members.to_vec();
    const VITE_BEFORE: &str = "providers/@voidzero-dev+vite-plus-core@0.1.24_@tsdown+css@0.22.14_@types+node@25.9.2_esbuild@0.28_a6c9f1199916a734d9b93ba95b0ddf69/node_modules/@voidzero-dev/vite-plus-core/";
    const VITE_AFTER: &str = "providers/@voidzero-dev+vite-plus-core@0.1.24_@types+node@25.9.2_esbuild@0.28.1_jiti@2.7.0_sass@1_648447c260f78446a04fe3ae6b71971f/node_modules/@voidzero-dev/vite-plus-core/";
    for (&entry, change) in entries.iter().zip(changes) {
        if change["entry"] != entry || change["before"] != members[entry] {
            return Err("eight dependency provider path changes");
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
        let path_is_exact = match entry {
            64 => {
                before_path
                    == "providers/@babel+types@7.29.8/node_modules/@babel/types/lib/index.d.ts"
                    && after_path
                        == "providers/@babel+types@7.29.11/node_modules/@babel/types/lib/index.d.ts"
            }
            65 => {
                before_path
                    == "providers/@babel+parser@7.29.8/node_modules/@babel/parser/typings/babel-parser.d.ts"
                    && after_path
                        == "providers/@babel+parser@7.29.13/node_modules/@babel/parser/typings/babel-parser.d.ts"
            }
            76..=81 => {
                before_path.strip_prefix(VITE_BEFORE) == after_path.strip_prefix(VITE_AFTER)
                    && before_path.starts_with(VITE_BEFORE)
            }
            _ => false,
        };
        if !path_is_exact || before[1..] != after[1..] {
            return Err("only the declared dependency provider paths may change");
        }
        current[entry] = change["after"].clone();
    }
    if record["wholeAfterRosterSha256"] != roster_hash(&current) {
        return Err("complete dependency provider successor roster");
    }
    Ok((record, current))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inputs() -> (Vec<Value>, Value, Value) {
        let mut first: Vec<Value> =
            serde_json::from_value(support::value("malformed-membership/part-00.json")).unwrap();
        let second: Vec<Value> =
            serde_json::from_value(support::value("malformed-membership/part-01.json")).unwrap();
        first.extend(second);
        let previous = super::super::load();
        let roster = super::super::successor(&first, &previous).unwrap();
        let qualification: Value =
            serde_json::from_slice(&fs::read(support::corpus().join(QUALIFICATION)).unwrap())
                .unwrap();
        (roster, previous, qualification)
    }

    #[test]
    fn exact_dependency_successor_changes_only_the_provider_path() {
        let (roster, previous, qualification) = inputs();
        let (record, current) = successor(&roster, &previous, &qualification).unwrap();
        assert_eq!(current.len(), roster.len());
        let changed = [64, 65, 76, 77, 78, 79, 80, 81];
        for (index, (before, after)) in roster.iter().zip(&current).enumerate() {
            if let Some(change) = changed.iter().position(|entry| *entry == index) {
                assert_eq!(after, &record["changes"][change]["after"]);
            } else {
                assert_eq!(after, before);
            }
        }
    }

    #[test]
    fn foreign_predecessors_bodies_and_paths_are_refused() {
        let (roster, previous, qualification) = inputs();
        let mut foreign_previous = previous.clone();
        foreign_previous["wholeAfterRosterSha256"] = json!(support::hash(b"foreign"));
        assert_eq!(
            successor(&roster, &foreign_previous, &qualification),
            Err("exact frozen dependency provider predecessor")
        );
        let mut foreign_qualification = qualification.clone();
        foreign_qualification["resolver"]["fixedPointSha256"] = json!(support::hash(b"foreign"));
        assert_eq!(
            successor(&roster, &previous, &foreign_qualification),
            Err("exact frozen dependency provider predecessor")
        );
        let mut body = roster.clone();
        body[64][2] = json!(support::hash(b"foreign"));
        assert_eq!(
            successor(&body, &previous, &qualification),
            Err("complete dependency provider predecessor roster")
        );
    }
}
