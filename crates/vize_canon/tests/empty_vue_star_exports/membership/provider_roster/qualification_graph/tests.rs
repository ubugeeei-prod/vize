use super::*;

struct Inputs {
    previous: Value,
    predecessor: Value,
    predecessor_bytes: Vec<u8>,
    update: Value,
    lock: Vec<u8>,
    package: Vec<u8>,
    parcel_manifest: Vec<u8>,
    cargo_workspace: Vec<u8>,
    cargo_lock: Vec<u8>,
}

fn inputs() -> Inputs {
    Inputs {
        previous: super::super::load(),
        predecessor: lockfile_update(),
        predecessor_bytes: fs::read(support::corpus().join(LOCKFILE_UPDATE)).unwrap(),
        update: parcel_update(),
        lock: fs::read(support::repository().join("pnpm-lock.yaml")).unwrap(),
        package: fs::read(support::repository().join("package.json")).unwrap(),
        parcel_manifest: fs::read(support::repository().join(PARCEL_MANIFEST)).unwrap(),
        cargo_workspace: fs::read(support::repository().join(CARGO_WORKSPACE)).unwrap(),
        cargo_lock: fs::read(support::repository().join(CARGO_LOCK)).unwrap(),
    }
}

impl Inputs {
    fn graph<'a>(&'a self, lock: &'a [u8]) -> ParcelQualificationGraph<'a> {
        ParcelQualificationGraph {
            lock,
            package: &self.package,
            parcel_manifest: &self.parcel_manifest,
            cargo_workspace: &self.cargo_workspace,
            cargo_lock: &self.cargo_lock,
        }
    }
}

fn verify(input: &Inputs, update: &Value, lock: &[u8]) -> Checked<()> {
    let graph = input.graph(lock);
    verify_parcel_update(&input.predecessor, &input.predecessor_bytes, update, &graph)
}

#[test]
fn exact_parcel_successor_binds_the_whole_current_graph() {
    let input = inputs();
    assert_eq!(verify(&input, &input.update, &input.lock), Ok(()));
    assert_eq!(
        authorize(&input.previous, &input.lock, &input.package),
        Ok(Some(input.update.clone()))
    );
}

#[test]
fn historical_lockfile_successor_remains_an_exact_predecessor() {
    let input = inputs();
    assert_eq!(
        support::hash(&input.predecessor_bytes),
        LOCKFILE_UPDATE_HASH
    );
    assert_eq!(
        input.update["previousProviderQualificationRecord"]["sha256"],
        LOCKFILE_UPDATE_HASH
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
            Err("exact current Parcel qualification graph bytes")
        );
    }

    let mut predecessor = input.predecessor.clone();
    predecessor["finalQualificationLock"]["sha256"] = json!(support::hash(b"foreign"));
    let graph = input.graph(&input.lock);
    assert_eq!(
        verify_parcel_update(
            &predecessor,
            &input.predecessor_bytes,
            &input.update,
            &graph,
        ),
        Err("exact frozen Parcel qualification successor")
    );

    let mut resolver = input.update.clone();
    resolver["resolvers"]["cargo"]["fixedPointSha256"] = json!(support::hash(b"foreign"));
    assert_eq!(
        verify(&input, &resolver, &input.lock),
        Err("exact Parcel resolver fixed points")
    );
}

#[test]
fn foreign_manifests_and_unreviewed_bytes_are_refused() {
    let input = inputs();
    let mut changed = input.update.clone();
    changed["parcelNativeManifest"]["sha256"] = json!(support::hash(b"foreign"));
    assert_eq!(
        verify(&input, &changed, &input.lock),
        Err("exact current Parcel qualification graph bytes")
    );

    let mut lock = input.lock.clone();
    *lock.first_mut().unwrap() ^= 1;
    let mut rehashed = input.update.clone();
    rehashed["finalQualificationLock"]["sha256"] = json!(support::hash(&lock));
    assert_eq!(
        verify(&input, &rehashed, &lock),
        Err("exact Parcel resolver fixed points")
    );
}
