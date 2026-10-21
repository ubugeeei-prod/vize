//! Full ordered malformed membership; reference bodies have separate provenance.

use super::support;
use serde_json::{Value, json};
use std::{
    fs,
    path::{Component, Path, PathBuf},
};

type Roots = [(&'static str, PathBuf); 3];
type Checked<T> = Result<T, &'static str>;

#[derive(Clone)]
struct Member {
    path: PathBuf,
    bytes: u64,
    sha256: String,
}

fn expected() -> (Value, Value) {
    let path = support::corpus().join("malformed-membership/manifest.json");
    let bytes = fs::read(path).unwrap();
    assert_eq!(
        support::hash(&bytes),
        "10dcbcf9ee2bc72fc205bdbcc6e2b487373b81265037e65e008b65c93d54975f"
    );
    let manifest: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(manifest["version"], 1);
    assert_eq!(manifest["case"], "malformed-refusal");
    assert_eq!(manifest["memberCount"], 86);
    assert_eq!(
        manifest["rootRoles"],
        json!(["native", "project", "providers"])
    );
    let parts = manifest["parts"].as_array().unwrap();
    assert_eq!(parts.len(), 2);
    let blobs = [
        "22eb745688ea02336b71df3e5f85779660ba9ae3",
        "5527050a3e6642e1f68159343a11a3563e05e862",
    ];
    let mut members = Vec::new();
    for (index, part) in parts.iter().enumerate() {
        assert_eq!(
            part["file"],
            format!("malformed-membership/part-{index:02}.json")
        );
        assert_eq!(part["mode"], "100644");
        assert_eq!(part["gitBlob"], blobs[index]);
        assert_eq!(part["start"], index * 43);
        assert_eq!(part["entries"], 43);
        let path = support::corpus().join(support::relative(part["file"].as_str().unwrap()));
        let metadata = fs::symlink_metadata(&path).unwrap();
        assert!(metadata.file_type().is_file());
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(metadata.permissions().mode() & 0o111, 0);
        }
        let bytes = fs::read(path).unwrap();
        assert_eq!(json!(bytes.len()), part["bytes"]);
        assert_eq!(support::hash(&bytes), part["sha256"].as_str().unwrap());
        let entries: Vec<Value> = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(entries.len(), 43);
        members.extend(entries);
    }
    assert_eq!(members.len(), 86);
    (manifest, json!(members))
}

fn pnpm(packet: &Value, role: &str) -> Checked<PathBuf> {
    let records = packet["genuineQualifiedProviders"]
        .as_array()
        .ok_or("provider records")?;
    let matching = records
        .iter()
        .filter(|record| record["role"]["id"] == role)
        .collect::<Vec<_>>();
    if matching.len() != 1 {
        return Err("unique authenticated manifest role required");
    }
    let record = &matching[0]["wholeManifestRead"];
    let text = record["wholeUtf8"].as_str().ok_or("whole manifest body")?;
    if record["bytes"] != json!(text.len()) || record["sha256"] != support::hash(text.as_bytes()) {
        return Err("manifest body custody");
    }
    let path = Path::new(record["path"].as_str().ok_or("manifest path")?)
        .canonicalize()
        .map_err(|_| "canonical manifest path")?;
    let roots = path
        .ancestors()
        .filter(|path| path.file_name() == Some(std::ffi::OsStr::new(".pnpm")))
        .map(Path::to_path_buf)
        .collect::<Vec<_>>();
    if roots.len() != 1 {
        return Err("one manifest-bound canonical pnpm ancestor required");
    }
    Ok(roots[0].clone())
}

fn roots(packet: &Value) -> Checked<Roots> {
    if packet["officialNativeCustody"]["phase"] != "OFFICIAL_NATIVE_RUNTIME_VALIDATED" {
        return Err("validated official executable required");
    }
    let executable = Path::new(
        packet["officialNativeCustody"]["executable"]
            .as_str()
            .ok_or("official executable")?,
    )
    .canonicalize()
    .map_err(|_| "canonical official executable")?;
    let config = Path::new(
        packet["generatedConfig"]["path"]
            .as_str()
            .ok_or("generated config")?,
    )
    .canonicalize()
    .map_err(|_| "canonical generated config")?;
    let vue = pnpm(packet, "vue-manifest")?;
    let vite = pnpm(packet, "vite-manifest")?;
    if vue != vite {
        return Err("Vue and Vite authenticated manifests must share the same pnpm root");
    }
    Ok([
        (
            "native",
            executable
                .parent()
                .ok_or("native lib parent")?
                .to_path_buf(),
        ),
        (
            "project",
            config.parent().ok_or("project parent")?.to_path_buf(),
        ),
        ("providers", vue),
    ])
}

fn captured(files: &Value) -> Checked<Vec<Member>> {
    files
        .as_array()
        .ok_or("whole captured files")?
        .iter()
        .map(|file| {
            if file.get("observedReadError").is_some() {
                return Err("whole file read failed");
            }
            let text = file["wholeUtf8"].as_str().ok_or("whole file bytes")?;
            let bytes = file["bytes"].as_u64().ok_or("file length")?;
            let sha256 = file["sha256"].as_str().ok_or("file SHA256")?;
            if bytes != u64::try_from(text.len()).map_err(|_| "length overflow")?
                || sha256 != support::hash(text.as_bytes())
            {
                return Err("whole file bytes/length/SHA256 mismatch");
            }
            let path = Path::new(file["path"].as_str().ok_or("file path")?)
                .canonicalize()
                .map_err(|_| "canonical captured file")?;
            Ok(Member {
                path,
                bytes,
                sha256: sha256.to_owned(),
            })
        })
        .collect()
}

fn normalize(roots: &Roots, members: &[Member]) -> Checked<Value> {
    let mut vector = Vec::new();
    for member in members {
        let (role, relative) = roots
            .iter()
            .find_map(|(role, root)| {
                member
                    .path
                    .strip_prefix(root)
                    .ok()
                    .map(|relative| (*role, relative))
            })
            .ok_or("foreign root")?;
        if relative.as_os_str().is_empty()
            || !relative
                .components()
                .all(|part| matches!(part, Component::Normal(_)))
        {
            return Err("exact rooted relative path required");
        }
        vector.push(json!([
            format!("{role}/{}", relative.to_string_lossy().replace('\\', "/")),
            member.bytes,
            member.sha256
        ]));
    }
    Ok(json!(vector))
}

fn compare(roots: &Roots, members: &[Member], expected: &Value) -> Checked<Value> {
    let actual = normalize(roots, members)?;
    if &actual != expected {
        let actual_members = actual.as_array().ok_or("normalized actual roster")?;
        let expected_members = expected.as_array().ok_or("normalized expected roster")?;
        let mismatch = actual_members
            .iter()
            .zip(expected_members)
            .position(|(actual, expected)| actual != expected)
            .unwrap_or_else(|| actual_members.len().min(expected_members.len()));
        eprintln!(
            "malformed membership mismatch at index {mismatch}; actual count {}; expected count {}; actual {}; expected {}",
            actual_members.len(),
            expected_members.len(),
            actual_members.get(mismatch).unwrap_or(&Value::Null),
            expected_members.get(mismatch).unwrap_or(&Value::Null),
        );
        return Err("complete ordered rooted-path/length/SHA256 roster mismatch");
    }
    Ok(actual)
}

pub(super) fn validate(packet: &Value, files: &Value) -> Value {
    assert_eq!(packet["case"]["id"], "malformed-refusal");
    assert_eq!(
        packet["case"]["nativeContract"],
        "projection-refusal-observation"
    );
    assert_eq!(
        packet["wholeNativeLoadedFileListingPartition"]["kind"],
        "single-malformed-case-syntax-prefix"
    );
    let (manifest, expected) = expected();
    let roots = roots(packet)
        .expect("only actual validated native/project/authenticated shared provider roots");
    let members =
        captured(files).expect("every actual captured body must have matching length and SHA256");
    let actual = compare(&roots, &members, &expected)
        .expect("all 86 exact members and bodies must retain their complete order");
    json!({"completeOrderedMembership":actual,"normalizedRoots":roots.iter().map(|(role,path)| json!([role,path])).collect::<Vec<_>>(),
        "referenceProvenance":manifest["provenance"],
        "authority":"current actual full 86-file read matched ordered reference; reference has explicit separate body provenance; no semantic-green credit"})
}

/// Exercise the actual normalizer/comparator with captured triplets and model roots.
/// This parser control has zero native, provider or actual loaded-file credit.
pub(super) fn controls() {
    let (_, expected) = expected();
    let temporary = tempfile::tempdir().unwrap();
    let base = temporary.path().canonicalize().unwrap();
    let providers = base.join("node_modules/.pnpm");
    let native =
        providers.join("@typescript+typescript-test/node_modules/@typescript/typescript-test/lib");
    let project = base.join("mirror");
    for root in [&native, &project, &providers] {
        fs::create_dir_all(root).unwrap();
    }
    let roots = [
        ("native", native),
        ("project", project),
        ("providers", providers),
    ];
    let members = expected
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| {
            let (role, relative) = entry[0].as_str().unwrap().split_once('/').unwrap();
            let root = roots.iter().find(|(name, _)| *name == role).unwrap();
            Member {
                path: root.1.join(relative),
                bytes: entry[1].as_u64().unwrap(),
                sha256: entry[2].as_str().unwrap().to_owned(),
            }
        })
        .collect::<Vec<_>>();
    assert_eq!(compare(&roots, &members, &expected).unwrap(), expected);
    let mut drop = members.clone();
    drop.pop();
    let mut add = members.clone();
    add.push(members[0].clone());
    let mut duplicate = members.clone();
    duplicate[1] = duplicate[0].clone();
    let mut reorder = members.clone();
    reorder.swap(0, 1);
    let mut foreign = members.clone();
    foreign[0].path = base.join("foreign/lib.es5.d.ts");
    let mut changed_length = members.clone();
    changed_length[0].bytes += 1;
    let mut changed_body = members.clone();
    changed_body[0].sha256 = support::hash(b"changed-body");
    assert_eq!(
        expected[73][0],
        "providers/vue@3.6.0-beta.10_typescript@6.0.3/node_modules/vue/dist/vue.d.mts"
    );
    assert_eq!(
        expected[81][0],
        "providers/@voidzero-dev+vite-plus-core@0.1.24_@tsdown+css@0.22.14_@types+node@25.9.2_esbuild@0.28_28053986ce44d27c9f2b7b250e1bd5a3/node_modules/@voidzero-dev/vite-plus-core/dist/vite/client.d.ts"
    );
    let mut lost_program = vec![members[73].clone(); 86];
    lost_program[85] = members[81].clone();
    for bad in [
        drop,
        add,
        duplicate,
        reorder,
        foreign,
        changed_length,
        changed_body,
        lost_program,
    ] {
        assert!(compare(&roots, &bad, &expected).is_err());
    }
    let file = base.join("body.ts");
    fs::write(&file, b"abc").unwrap();
    let record = support::observed_custody(&file);
    assert!(captured(&json!([record.clone()])).is_ok());
    let mut changed = record;
    changed["wholeUtf8"] = json!("abd");
    assert!(captured(&json!([changed])).is_err());
}
