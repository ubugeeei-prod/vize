import {
  fixedMerge,
  repository,
  apiRoot,
} from "../../../tools/support/release/typed_defaults/custody.mjs";

/** Inert provider metadata for refusal laws; no product or publication evidence. */
export function officialFixture() {
  const options = {
    cut: "c".repeat(40),
    head: "d".repeat(40),
    tag: "v0.445.0",
    sourcePr: "8577",
    run: "38111825534",
  };
  const oid = "a".repeat(40);
  const rows = new Map();
  const put = (route, value) => rows.set(`${apiRoot}/${route}`, value);
  put(`git/ref/tags/${options.tag}`, {
    ref: `refs/tags/${options.tag}`,
    object: { type: "tag", sha: oid },
  });
  put(`git/tags/${oid}`, {
    sha: oid,
    tag: options.tag,
    object: { type: "commit", sha: options.head },
    message: `Release ${options.tag}\nRelease-Mode: pinned\nRelease-PR: #${options.sourcePr}\nValidated-run: ${options.run}\nValidated-head: ${options.head}\nValidated-base: ${options.cut}\n`,
  });
  put(`pulls/${options.sourcePr}`, {
    number: Number(options.sourcePr),
    state: "closed",
    merged: false,
    merged_at: null,
    head: { sha: options.head, ref: `release/${options.tag}`, repo: { full_name: repository } },
    base: { ref: "main", repo: { full_name: repository } },
    body: `<!-- vize-release-pin: immutable-v1 -->\n<!-- vize-release-pin-cut: ${options.cut} -->\n<!-- vize-release-pin-head: ${options.head} -->\n<!-- vize-release-integration: 8578 -->\n`,
  });
  put(`actions/runs/${options.run}`, {
    id: Number(options.run),
    run_attempt: 1,
    repository: { full_name: repository },
    head_repository: { full_name: repository },
    head_sha: options.head,
    head_branch: `release/${options.tag}`,
    event: "workflow_dispatch",
    display_title: `Pinned Release ${options.tag} PR #${options.sourcePr} @ ${options.head}`,
    path: ".github/workflows/release.yml",
    status: "completed",
    conclusion: "success",
  });
  put(`actions/runs/${options.run}/jobs?filter=latest&per_page=100&page=1`, {
    total_count: 1,
    jobs: [
      {
        id: 12,
        name: "inert Release job",
        run_id: Number(options.run),
        run_attempt: 1,
        head_sha: options.head,
        status: "completed",
        conclusion: "success",
      },
    ],
  });
  put(`releases/tags/${options.tag}`, {
    id: 123,
    tag_name: options.tag,
    draft: false,
    prerelease: false,
    published_at: "2026-10-11T00:00:00Z",
  });
  put(`git/commits/${options.head}`, {
    sha: options.head,
    parents: [{ sha: options.cut }],
    tree: { sha: "b".repeat(40) },
  });
  put("pulls/8565", {
    merged: true,
    merge_commit_sha: fixedMerge,
    base: { repo: { full_name: repository } },
    head: { repo: { full_name: repository } },
  });
  put(`compare/${fixedMerge}...${options.head}`, {
    base_commit: { sha: fixedMerge },
    merge_base_commit: { sha: fixedMerge },
    status: "ahead",
  });
  return {
    options,
    rows,
    read: async (url) => {
      if (!rows.has(url)) throw new Error(`unplanned inert provider read: ${url}`);
      return structuredClone(rows.get(url));
    },
  };
}
