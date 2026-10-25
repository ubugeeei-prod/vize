use std::{
    future::Future,
    pin::pin,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    task::{Context, Poll, Wake, Waker},
};

use serde_json::json;

use super::{AsyncGate, merge_projects};
use crate::api::{ProjectHandle, ProjectResponse, SnapshotChanges};

fn project(id: &str, root_file: &str) -> ProjectResponse {
    ProjectResponse {
        id: ProjectHandle::from(id),
        config_file_name: format!("{id}/tsconfig.json"),
        compiler_options: json!({}),
        root_files: vec![root_file.into()],
    }
}

fn ids(projects: &[ProjectResponse]) -> Vec<&str> {
    projects.iter().map(|project| project.id.as_str()).collect()
}

#[test]
fn update_without_project_changes_keeps_the_base_project_list() {
    let merged = merge_projects(
        vec![project("app", "a.ts"), project("lib", "b.ts")],
        vec![],
        None,
    );

    assert_eq!(ids(&merged), ["app", "lib"]);
}

#[test]
fn updated_projects_replace_in_place_and_new_ones_append() {
    let merged = merge_projects(
        vec![project("app", "a.ts"), project("lib", "b.ts")],
        vec![project("tools", "t.ts"), project("app", "a2.ts")],
        None,
    );

    assert_eq!(ids(&merged), ["app", "lib", "tools"]);
    assert_eq!(merged[0].root_files, ["a2.ts"]);
}

#[test]
fn removed_projects_leave_the_list_even_when_nothing_replaces_them() {
    let changes = SnapshotChanges {
        removed_projects: vec![ProjectHandle::from("lib")],
        ..SnapshotChanges::default()
    };

    let merged = merge_projects(
        vec![project("app", "a.ts"), project("lib", "b.ts")],
        vec![],
        Some(&changes),
    );

    assert_eq!(ids(&merged), ["app"]);
}

struct CountingWaker(AtomicUsize);

impl Wake for CountingWaker {
    fn wake(self: Arc<Self>) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}

#[test]
fn gate_admits_one_holder_and_wakes_the_next_on_release() {
    let gate = AsyncGate::default();
    let wakes = Arc::new(CountingWaker(AtomicUsize::new(0)));
    let waker = Waker::from(Arc::clone(&wakes));
    let mut cx = Context::from_waker(&waker);

    let mut first = pin!(gate.acquire());
    let Poll::Ready(permit) = first.as_mut().poll(&mut cx) else {
        panic!("an idle gate admits immediately");
    };
    let mut second = pin!(gate.acquire());
    assert!(second.as_mut().poll(&mut cx).is_pending());
    assert_eq!(wakes.0.load(Ordering::SeqCst), 0);

    drop(permit);

    assert_eq!(wakes.0.load(Ordering::SeqCst), 1);
    assert!(second.as_mut().poll(&mut cx).is_ready());
}
