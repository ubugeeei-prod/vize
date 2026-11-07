use super::*;

struct Inputs {
    previous: Value,
    predecessor: Value,
    predecessor_bytes: Vec<u8>,
    update: Value,
    lock: Vec<u8>,
    package: Vec<u8>,
    fresco_manifest: Vec<u8>,
}

fn inputs() -> Inputs {
    Inputs {
        previous: super::super::load(),
        predecessor: tooling_update(),
        predecessor_bytes: fs::read(support::corpus().join(TOOLING_UPDATE)).unwrap(),
        update: fresco_update(),
        lock: fs::read(support::repository().join("pnpm-lock.yaml")).unwrap(),
        package: fs::read(support::repository().join("package.json")).unwrap(),
        fresco_manifest: fs::read(support::repository().join(FRESCO_MANIFEST)).unwrap(),
    }
}

impl Inputs {
    fn graph<'a>(&'a self, lock: &'a [u8]) -> FrescoQualificationGraph<'a> {
        FrescoQualificationGraph {
            lock,
            package: &self.package,
            fresco_manifest: &self.fresco_manifest,
        }
    }
}

fn verify(input: &Inputs, update: &Value, lock: &[u8]) -> Checked<()> {
    let graph = input.graph(lock);
    verify_fresco_update(&input.predecessor, &input.predecessor_bytes, update, &graph)
}

#[test]
fn exact_fresco_successor_binds_the_whole_current_graph() {
    let input = inputs();
    assert_eq!(verify(&input, &input.update, &input.lock), Ok(()));
    assert_eq!(
        authorize(&input.previous, &input.lock, &input.package),
        Ok(Some(input.update.clone()))
    );
}

#[test]
fn historical_tooling_successor_remains_an_exact_predecessor() {
    let input = inputs();
    assert_eq!(support::hash(&input.predecessor_bytes), TOOLING_UPDATE_HASH);
    assert_eq!(
        input.update["previousProviderQualificationRecord"]["sha256"],
        TOOLING_UPDATE_HASH
    );
    assert_eq!(
        input.update["previousQualificationLock"],
        input.predecessor["finalQualificationLock"]
    );
}

#[test]
fn changed_graph_predecessor_and_fixed_point_are_refused() {
    let input = inputs();
    for changed in [
        b"foreign lock".as_slice(),
        input.lock.get(..input.lock.len() - 1).unwrap(),
    ] {
        assert_eq!(
            verify(&input, &input.update, changed),
            Err("exact current Fresco qualification graph bytes")
        );
    }

    let mut predecessor = input.predecessor.clone();
    predecessor["finalQualificationLock"]["sha256"] = json!(support::hash(b"foreign"));
    let graph = input.graph(&input.lock);
    assert_eq!(
        verify_fresco_update(
            &predecessor,
            &input.predecessor_bytes,
            &input.update,
            &graph,
        ),
        Err("exact frozen Fresco qualification successor")
    );

    let mut resolver = input.update.clone();
    resolver["resolver"]["fixedPointSha256"] = json!(support::hash(b"foreign"));
    assert_eq!(
        verify(&input, &resolver, &input.lock),
        Err("exact Fresco resolver fixed point")
    );
}

#[test]
fn foreign_manifests_and_unreviewed_bytes_are_refused() {
    let input = inputs();
    let mut changed = input.update.clone();
    changed["frescoManifest"]["sha256"] = json!(support::hash(b"foreign"));
    assert_eq!(
        verify(&input, &changed, &input.lock),
        Err("exact current Fresco qualification graph bytes")
    );

    let mut lock = input.lock.clone();
    *lock.first_mut().unwrap() ^= 1;
    let mut rehashed = input.update.clone();
    rehashed["finalQualificationLock"]["sha256"] = json!(support::hash(&lock));
    assert_eq!(
        verify(&input, &rehashed, &lock),
        Err("exact Fresco resolver fixed point")
    );

    let nested = String::from_utf8(input.lock.clone())
        .unwrap()
        .replacen("\n  npm/fresco:\n", "\n    npm/fresco:\n", 1)
        .into_bytes();
    let mut rebound = input.update.clone();
    let nested_hash = support::hash(&nested);
    rebound["finalQualificationLock"]["bytes"] = json!(nested.len());
    rebound["finalQualificationLock"]["sha256"] = json!(nested_hash);
    rebound["resolver"]["fixedPointSha256"] = json!(nested_hash);
    assert_eq!(
        verify(&input, &rebound, &nested),
        Err("one exact Fresco importer")
    );
}
