use super::super::tests::Repo;
use super::*;

fn commit(repo: &Repo, tree: &str, parents: &[&str], message: &str) -> String {
    let mut args = vec!["commit-tree", tree];
    for parent in parents {
        args.extend(["-p", parent]);
    }
    args.extend(["-m", message]);
    let sha = github::git(&args, &repo.work).unwrap();
    github::git(
        &["push", "origin", &format!("{sha}:refs/heads/law-{sha}")],
        &repo.work,
    )
    .unwrap();
    sha
}

fn fixture() -> (Repo, String, String, String, String) {
    let repo = Repo::new();
    let base = repo.commit(&[("law.txt", "base\n")]);
    let head = repo.commit(&[("law.txt", "head\n")]);
    let tree = github::git(&["rev-parse", &format!("{head}^{{tree}}")], &repo.work).unwrap();
    let merge = commit(&repo, &tree, &[&base, &head], "authored PR merge");
    (repo, base, head, tree, merge)
}

fn traversal(repo: &Repo, sha: &str) -> String {
    github::git(&["rev-list", "--parents", "-n", "1", sha], &repo.work).unwrap()
}

#[test]
fn shallow_boundary_does_not_erase_authentic_ordered_pr_parents() {
    let (repo, base, head, _, merge) = fixture();
    fs::write(repo.work.join(".git/shallow"), format!("{merge}\n")).unwrap();
    assert_eq!(traversal(&repo, &merge), merge);
    println!("shallow control: candidate={merge}, observed={merge}, expected={base} {head}");
    candidate_parents(&merge, &base, &head, false, &repo.work).unwrap();
    assert!(candidate_parents(&merge, &base, &head, true, &repo.work).is_err());
}

#[test]
fn graft_does_not_erase_authentic_ordered_pr_parents() {
    let (repo, base, head, _, merge) = fixture();
    fs::write(
        repo.work.join(".git/info/grafts"),
        format!("{merge} {base}\n"),
    )
    .unwrap();
    assert_eq!(traversal(&repo, &merge), format!("{merge} {base}"));
    println!("graft control: candidate={merge}, observed={merge} {base}, expected={base} {head}");
    candidate_parents(&merge, &base, &head, false, &repo.work).unwrap();
    assert!(candidate_parents(&merge, &base, &head, true, &repo.work).is_err());
}

#[test]
fn replacement_does_not_erase_authentic_ordered_pr_parents() {
    let (repo, base, head, tree, merge) = fixture();
    let replacement = commit(&repo, &tree, &[&base], "replacement sole-parent commit");
    github::git(&["replace", &merge, &replacement], &repo.work).unwrap();
    assert_eq!(traversal(&repo, &merge), format!("{merge} {base}"));
    println!(
        "replace control: candidate={merge}, replacement={replacement}, expected={base} {head}"
    );
    candidate_parents(&merge, &base, &head, false, &repo.work).unwrap();
    assert!(candidate_parents(&merge, &base, &head, true, &repo.work).is_err());
}

#[test]
fn graft_cannot_make_wrong_raw_parents_qualify() {
    let (repo, base, head, tree, _) = fixture();
    let wrong = commit(&repo, &tree, &[&base], "wrong PR raw sole parent");
    fs::write(
        repo.work.join(".git/info/grafts"),
        format!("{wrong} {base} {head}\n"),
    )
    .unwrap();
    assert_eq!(traversal(&repo, &wrong), format!("{wrong} {base} {head}"));
    println!(
        "graft spoof control: candidate={wrong}, raw sole parent={base}, virtual second={head}"
    );
    assert!(candidate_parents(&wrong, &base, &head, false, &repo.work).is_err());
    candidate_parents(&wrong, &base, &head, true, &repo.work).unwrap();
}

#[test]
fn replacement_cannot_make_wrong_raw_parents_qualify() {
    let (repo, base, head, tree, merge) = fixture();
    let wrong = commit(&repo, &tree, &[&base], "wrong raw sole parent");
    github::git(&["replace", &wrong, &merge], &repo.work).unwrap();
    assert_eq!(traversal(&repo, &wrong), format!("{wrong} {base} {head}"));
    println!("replace spoof control: candidate={wrong}, replacement={merge}, raw sole={base}");
    assert!(candidate_parents(&wrong, &base, &head, false, &repo.work).is_err());
    candidate_parents(&wrong, &base, &head, true, &repo.work).unwrap();
}

#[test]
fn replacement_cannot_hide_changed_complete_refresh_tree() {
    let (repo, base, head, _, event) = fixture();
    let tree = github::git(&["rev-parse", &format!("{base}^{{tree}}")], &repo.work).unwrap();
    let changed = commit(&repo, &tree, &[&base, &head], "changed raw refresh tree");
    github::git(&["replace", &changed, &event], &repo.work).unwrap();
    assert_eq!(
        github::git(&["rev-parse", &format!("{changed}^{{tree}}")], &repo.work).unwrap(),
        github::git(&["rev-parse", &format!("{event}^{{tree}}")], &repo.work).unwrap()
    );
    println!("tree spoof control: event={event}, current={changed}, raw tree={tree}");
    assert!(current_pr_snapshot(&event, &changed, &base, &head, true, &repo.work).is_err());
}

#[test]
fn raw_parent_count_order_and_queue_shape_remain_strict() {
    let (repo, base, head, tree, event) = fixture();
    candidate_parents(&event, &base, &head, false, &repo.work).unwrap();
    for parents in [
        vec![head.as_str(), base.as_str()],
        vec![base.as_str()],
        vec![base.as_str(), head.as_str(), event.as_str()],
    ] {
        let wrong = commit(&repo, &tree, &parents, "authored wrong PR parent vector");
        assert!(candidate_parents(&wrong, &base, &head, false, &repo.work).is_err());
    }
    let queue = commit(&repo, &tree, &[&base], "authored queue candidate");
    candidate_parents(&queue, &base, &head, true, &repo.work).unwrap();
    assert!(candidate_parents(&queue, &head, &head, true, &repo.work).is_err());
    assert!(candidate_parents(&event, &base, &head, true, &repo.work).is_err());
}

#[test]
fn parent_refusal_retains_expected_observed_identity_and_refresh_stage() {
    let (repo, base, head, tree, event) = fixture();
    let wrong = commit(&repo, &tree, &[&base], "wrong refresh parent count");
    let error = candidate_parents(&wrong, &base, &head, false, &repo.work).unwrap_err();
    for field in [
        "observed=",
        "expected=",
        "stage=candidate identity",
        &wrong,
        &base,
        &head,
    ] {
        assert!(error.contains(field), "missing {field}: {error}");
    }
    let error = current_pr_snapshot(&event, &wrong, &base, &head, true, &repo.work).unwrap_err();
    assert!(error.contains("stage=current PR refresh"), "{error}");
}
