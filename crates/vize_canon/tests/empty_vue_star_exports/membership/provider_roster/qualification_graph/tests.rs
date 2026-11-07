use super::*;

struct Inputs {
    previous: Value,
    predecessor: Value,
    predecessor_bytes: Vec<u8>,
    update: Value,
    lock: Vec<u8>,
    package: Vec<u8>,
    workspace: Vec<u8>,
    babel_manifest: Vec<u8>,
    eslint_manifest: Vec<u8>,
    oxlint_manifest: Vec<u8>,
    prettier_manifest: Vec<u8>,
}

fn inputs() -> Inputs {
    Inputs {
        previous: super::super::load(),
        predecessor: parcel_update(),
        predecessor_bytes: fs::read(support::corpus().join(PARCEL_UPDATE)).unwrap(),
        update: tooling_update(),
        lock: fs::read(support::repository().join("pnpm-lock.yaml")).unwrap(),
        package: fs::read(support::repository().join("package.json")).unwrap(),
        workspace: fs::read(support::repository().join(WORKSPACE)).unwrap(),
        babel_manifest: fs::read(support::repository().join(BABEL_MANIFEST)).unwrap(),
        eslint_manifest: fs::read(support::repository().join(ESLINT_MANIFEST)).unwrap(),
        oxlint_manifest: fs::read(support::repository().join(OXLINT_MANIFEST)).unwrap(),
        prettier_manifest: fs::read(support::repository().join(PRETTIER_MANIFEST)).unwrap(),
    }
}

impl Inputs {
    fn graph<'a>(&'a self, lock: &'a [u8]) -> ToolingQualificationGraph<'a> {
        ToolingQualificationGraph {
            lock,
            package: &self.package,
            workspace: &self.workspace,
            babel_manifest: &self.babel_manifest,
            eslint_manifest: &self.eslint_manifest,
            oxlint_manifest: &self.oxlint_manifest,
            prettier_manifest: &self.prettier_manifest,
        }
    }
}

fn verify(input: &Inputs, update: &Value, lock: &[u8]) -> Checked<()> {
    let graph = input.graph(lock);
    verify_tooling_update(&input.predecessor, &input.predecessor_bytes, update, &graph)
}

#[test]
fn exact_tooling_successor_binds_the_whole_current_graph() {
    let input = inputs();
    assert_eq!(verify(&input, &input.update, &input.lock), Ok(()));
    assert_eq!(
        authorize(&input.previous, &input.lock, &input.package),
        Ok(Some(input.update.clone()))
    );
}

#[test]
fn historical_parcel_successor_remains_an_exact_predecessor() {
    let input = inputs();
    assert_eq!(support::hash(&input.predecessor_bytes), PARCEL_UPDATE_HASH);
    assert_eq!(
        input.update["previousProviderQualificationRecord"]["sha256"],
        PARCEL_UPDATE_HASH
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
            Err("exact current JavaScript tooling qualification graph bytes")
        );
    }

    let mut predecessor = input.predecessor.clone();
    predecessor["finalQualificationLock"]["sha256"] = json!(support::hash(b"foreign"));
    let graph = input.graph(&input.lock);
    assert_eq!(
        verify_tooling_update(
            &predecessor,
            &input.predecessor_bytes,
            &input.update,
            &graph,
        ),
        Err("exact frozen JavaScript tooling qualification successor")
    );

    let mut resolver = input.update.clone();
    resolver["resolver"]["fixedPointSha256"] = json!(support::hash(b"foreign"));
    assert_eq!(
        verify(&input, &resolver, &input.lock),
        Err("exact JavaScript tooling resolver fixed point")
    );
}

#[test]
fn foreign_manifests_and_unreviewed_bytes_are_refused() {
    let input = inputs();
    let mut changed = input.update.clone();
    changed["oxlintManifest"]["sha256"] = json!(support::hash(b"foreign"));
    assert_eq!(
        verify(&input, &changed, &input.lock),
        Err("exact current JavaScript tooling qualification graph bytes")
    );

    let mut lock = input.lock.clone();
    *lock.first_mut().unwrap() ^= 1;
    let mut rehashed = input.update.clone();
    rehashed["finalQualificationLock"]["sha256"] = json!(support::hash(&lock));
    assert_eq!(
        verify(&input, &rehashed, &lock),
        Err("exact JavaScript tooling resolver fixed point")
    );
}
