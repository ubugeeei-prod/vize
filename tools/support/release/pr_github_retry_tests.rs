use super::*;

#[test]
fn only_read_only_gh_api_requests_are_retried() {
    for args in [
        &[
            "api",
            "repos/o/r/actions/runs/1/jobs?filter=all&per_page=100&page=1",
        ][..],
        &["api", "repos/o/r/pulls/1", "--jq", ".head.sha"][..],
        &["api", "--paginate", "repos/o/r/issues"][..],
    ] {
        assert!(is_read_only_gh_api("gh", args), "{args:?}");
    }
    for (program, args) in [
        ("git", &["api", "repos/o/r"][..]),
        ("gh", &["pr", "view", "1"][..]),
        ("gh", &["api", "-X", "PATCH", "repos/o/r/pulls/1"][..]),
        ("gh", &["api", "-XPOST", "repos/o/r/issues"][..]),
        (
            "gh",
            &["api", "--method=PUT", "repos/o/r/pulls/1/merge"][..],
        ),
        ("gh", &["api", "repos/o/r/issues", "-f", "title=x"][..]),
        ("gh", &["api", "repos/o/r/issues", "-F", "n=1"][..]),
        (
            "gh",
            &["api", "repos/o/r/issues", "--raw-field", "body=x"][..],
        ),
        ("gh", &["api", "graphql", "--input", "query.json"][..]),
    ] {
        assert!(!is_read_only_gh_api(program, args), "{program} {args:?}");
    }
}

#[test]
fn only_server_errors_are_transient() {
    for stderr in [
        "{\n  \"message\": \"Server Error\"\n}\ngh: Server Error (HTTP 502)",
        "gh: Service Unavailable (HTTP 503)",
        "gh: Gateway Timeout (HTTP 504)",
        "gh: Internal Server Error (HTTP 500)",
    ] {
        assert!(is_transient_server_error(stderr), "{stderr}");
    }
    for stderr in [
        "gh: Not Found (HTTP 404)",
        "gh: Resource not accessible by integration (HTTP 403)",
        "gh: Validation Failed (HTTP 422)",
        "error connecting to api.github.com",
    ] {
        assert!(!is_transient_server_error(stderr), "{stderr}");
    }
}
