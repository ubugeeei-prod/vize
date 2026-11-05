use super::*;

/// Exercise the actual normalizer/comparator with captured triplets and model roots.
/// This parser control has zero native, provider or actual loaded-file credit.
pub(crate) fn controls() {
    let (_, expected) = expected(false);
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
