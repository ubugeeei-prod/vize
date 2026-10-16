//! Contexts admitted before final freeze are captured; later initializers cannot serve.
use super::*;
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

#[test]
fn final_freeze_keeps_raw_published_owner_after_ordinary_lookup_hides_it() {
    let root = tempfile::tempdir().unwrap();
    let owner = Arc::new(ServerState::new());
    let context = ProjectContext::new(root.path().to_path_buf());
    let state = context.initialize(&owner).unwrap();
    context.mark_retired_for_exit();
    assert!(context.initialized_state().is_none());
    assert!(Arc::ptr_eq(
        &context.initialized_state_for_exit().unwrap(),
        &state
    ));
    assert!(state.project_context_retired());
    assert!(context.initialize(&owner).is_none());
}

struct Release(PathBuf);
impl Drop for Release {
    fn drop(&mut self) {
        let _ = std::fs::write(&self.0, "1");
    }
}

#[test]
fn initializer_entered_before_freeze_cannot_return_owner_published_after_freeze() {
    let root = tempfile::tempdir().unwrap();
    let started = root.path().join("started");
    let release = root.path().join("release");
    std::fs::write(root.path().join("vite.config.mjs"), "import {existsSync,writeFileSync} from 'node:fs'; writeFileSync(new URL('./started',import.meta.url),'1'); while(!existsSync(new URL('./release',import.meta.url))) await new Promise(r=>setTimeout(r,10)); export default {};\n").unwrap();
    let _release = Release(release.clone());
    let owner = Arc::new(ServerState::new());
    let context = Arc::new(ProjectContext::new(root.path().to_path_buf()));
    let pending = context.clone();
    let worker = std::thread::spawn(move || pending.initialize(&owner));
    let deadline = Instant::now() + Duration::from_secs(5);
    while !started.exists() {
        assert!(
            Instant::now() < deadline,
            "actual config initializer did not enter"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
    context.mark_retired_for_exit();
    assert!(context.initialized_state_for_exit().is_none());
    std::fs::write(&release, "1").unwrap();
    assert!(
        worker.join().unwrap().is_none(),
        "retired initializer cannot expose a native request owner"
    );
    let late = context.initialized_state_for_exit().unwrap();
    assert!(late.project_context_retired());
    assert!(!late.has_corsa_bridge());
}
