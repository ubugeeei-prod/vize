use super::*;

const BODY: &str = "interface Array<T> { length: number }\n";

fn fixture() -> (tempfile::TempDir, PathBuf, Vec<Asset>, String) {
    let owner = tempfile::tempdir().unwrap();
    let root = owner.path().canonicalize().unwrap().join("lib");
    fs::create_dir(&root).unwrap();
    let native = root.join("tsc");
    fs::write(&native, "image").unwrap();
    fs::write(root.join("lib.d.ts"), BODY).unwrap();
    let assets = vec![Asset {
        name: "lib.d.ts".into(),
        bytes: BODY.len() as u64,
        sha256: encode(Sha256::digest(BODY.as_bytes())),
    }];
    (owner, native, assets, encode(Sha256::digest(b"image")))
}

#[test]
fn complete_adjacent_manifest_retains_whole_bytes_and_paths() {
    let (_owner, native, assets, image) = fixture();
    let libraries = inspect_manifest(&native, 5, &image, &assets).unwrap();
    assert_eq!(libraries.root, native.parent().unwrap());
    assert_eq!(libraries.sources.len(), 1);
    assert_eq!(
        libraries.sources[0].path,
        native.parent().unwrap().join("lib.d.ts")
    );
    assert_eq!(libraries.sources[0].body, BODY);
    assert!(
        inspect(&native).is_none(),
        "unit fake image is never the packaged provider"
    );
}

#[test]
fn wrong_image_or_missing_changed_extra_assets_never_claim_the_package() {
    let (_owner, native, assets, image) = fixture();
    assert!(inspect_manifest(&native, 5, "wrong", &assets).is_none());
    assert!(inspect_manifest(&native, 6, &image, &assets).is_none());
    let path = native.parent().unwrap().join("lib.d.ts");
    fs::write(&path, BODY.replace("number", "string")).unwrap();
    assert!(inspect_manifest(&native, 5, &image, &assets).is_none());
    fs::write(&path, BODY).unwrap();
    let extra = native.parent().unwrap().join("lib.extra.d.ts");
    fs::write(&extra, "type Poison = true;\n").unwrap();
    assert!(inspect_manifest(&native, 5, &image, &assets).is_none());
    fs::remove_file(&extra).unwrap();
    fs::remove_file(&path).unwrap();
    assert!(inspect_manifest(&native, 5, &image, &assets).is_none());
}

#[cfg(unix)]
#[test]
fn linked_assets_and_noncanonical_executable_spellings_are_unknown() {
    use std::os::unix::fs::symlink;
    let (_owner, native, assets, image) = fixture();
    let root = native.parent().unwrap();
    let outside = root.parent().unwrap().join("outside.d.ts");
    fs::write(&outside, BODY).unwrap();
    fs::remove_file(root.join("lib.d.ts")).unwrap();
    symlink(&outside, root.join("lib.d.ts")).unwrap();
    assert!(inspect_manifest(&native, 5, &image, &assets).is_none());
    assert_eq!(fs::read_link(root.join("lib.d.ts")).unwrap(), outside);
    let alias = root.parent().unwrap().join("linked");
    symlink(root, &alias).unwrap();
    assert!(inspect_manifest(&alias.join("tsc"), 5, &image, &assets).is_none());
}

#[test]
fn all_116_held_assets_including_lib_d_ts_are_pinned_without_a_body_download() {
    let assets: Vec<Asset> = serde_json::from_str(ASSETS).unwrap();
    assert_eq!(assets.len(), 116);
    assert_eq!(assets[0].name, "lib.d.ts");
    assert_eq!(assets[0].bytes, 953);
    assert_eq!(
        assets[0].sha256,
        "0e6477e5049e579cc5de741a70ac91235252edc0468b2e569f22de9db3357786"
    );
    assert_eq!(
        assets.iter().map(|asset| asset.bytes).sum::<u64>(),
        3_870_196
    );
    assert!(assets.windows(2).all(|pair| pair[0].name < pair[1].name));
    let mut extra: serde_json::Value = serde_json::from_str(ASSETS).unwrap();
    extra[0]["unknown"] = true.into();
    assert!(serde_json::from_value::<Vec<Asset>>(extra).is_err());
}
