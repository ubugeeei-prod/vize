import type { PublishedRelease } from "../../../tools/support/release/github_release_notes.ts";

const repository = "owner/repo";
const tag = "v0.457.0";
const [object, head, cut, integrationHead, merge, tree, gate] = "abcdef0"
  .split("")
  .map((x) => x.repeat(40));

export const publishedIntegrationIdentity = { repository, tag, object, head, merge };

export const release = (
  tag_name: string,
  overrides: Partial<PublishedRelease> = {},
): PublishedRelease => ({
  tag_name,
  draft: false,
  prerelease: tag_name.includes("-"),
  published_at: "2026-10-17T00:00:00Z",
  ...overrides,
});

export const metadataPage = (
  releases: PublishedRelease[],
  hasNextPage = false,
  endCursor: string | null = null,
) => ({
  data: {
    repository: {
      releases: {
        nodes: releases.map((item) => ({
          tagName: item.tag_name,
          isDraft: item.draft,
          isPrerelease: item.prerelease,
          publishedAt: item.published_at,
        })),
        pageInfo: { hasNextPage, endCursor },
      },
    },
  },
});

export const publishedIntegrationFixture = () => ({
  reference: { ref: `refs/tags/${tag}`, object: { type: "tag", sha: object } },
  annotated: {
    sha: object,
    tag,
    object: { type: "commit", sha: head },
    message: `Release ${tag}\n\nRelease-Mode: pinned\nRelease-PR: #72\nValidated-run: 19\nValidated-head: ${head}\nValidated-base: ${cut}\nIntegration-PR: #73\nIntegration-head: ${integrationHead}\nIntegration-merge: ${merge}\nIntegration-parent: ${cut}\nProtected-candidate: ${gate}\n`,
  },
  pr: {
    number: 73,
    merged: true,
    state: "closed",
    merged_at: "2026-10-17T00:00:00Z",
    merge_commit_sha: merge,
    base: { ref: "main", repo: { full_name: repository } },
    head: { sha: integrationHead, repo: { full_name: repository } },
    body: `<!-- vize-release-pin-source: 72 -->\n<!-- vize-release-pin-cut: ${cut} -->\n<!-- vize-release-pin-head: ${head} -->\n<!-- vize-release-pin-tag: ${tag} -->`,
  },
  h: { sha: head, parents: [{ sha: cut }], commit: { tree: { sha: tree } } },
  m: {
    sha: merge,
    parents: [{ sha: cut }],
    commit: { tree: { sha: tree }, verification: { verified: true, reason: "valid" } },
  },
});

export const unprovedIntegrationMutations = [
  (f: ReturnType<typeof publishedIntegrationFixture>) => {
    f.reference.object.type = "commit";
  },
  (f: ReturnType<typeof publishedIntegrationFixture>) => {
    f.reference.ref += "-other";
  },
  (f: ReturnType<typeof publishedIntegrationFixture>) => {
    f.annotated.sha = gate;
  },
  (f: ReturnType<typeof publishedIntegrationFixture>) => {
    f.annotated.object.sha = gate;
  },
  (f: ReturnType<typeof publishedIntegrationFixture>) => {
    f.annotated.tag = "v0.456.0";
  },
  (f: ReturnType<typeof publishedIntegrationFixture>) => {
    f.annotated.message += "Release-PR: #72\n";
  },
  (f: ReturnType<typeof publishedIntegrationFixture>) => {
    f.annotated.message = f.annotated.message.replace("Validated-run: 19", "Validated-run: 0");
  },
  (f: ReturnType<typeof publishedIntegrationFixture>) => {
    f.annotated.message = f.annotated.message.replace(
      `Integration-parent: ${cut}`,
      `Integration-parent: ${gate}`,
    );
  },
  (f: ReturnType<typeof publishedIntegrationFixture>) => {
    f.pr.number = 74;
  },
  (f: ReturnType<typeof publishedIntegrationFixture>) => {
    f.pr.merged = false;
  },
  (f: ReturnType<typeof publishedIntegrationFixture>) => {
    f.pr.state = "open";
  },
  (f: ReturnType<typeof publishedIntegrationFixture>) => {
    f.pr.merge_commit_sha = gate;
  },
  (f: ReturnType<typeof publishedIntegrationFixture>) => {
    f.pr.head.sha = gate;
  },
  (f: ReturnType<typeof publishedIntegrationFixture>) => {
    f.pr.base.ref = "other";
  },
  (f: ReturnType<typeof publishedIntegrationFixture>) => {
    f.pr.base.repo.full_name = "foreign/repo";
  },
  (f: ReturnType<typeof publishedIntegrationFixture>) => {
    f.pr.head.repo.full_name = "foreign/repo";
  },
  (f: ReturnType<typeof publishedIntegrationFixture>) => {
    f.pr.body = f.pr.body.replace(head, gate);
  },
  (f: ReturnType<typeof publishedIntegrationFixture>) => {
    f.pr.body += "\n<!-- vize-release-pin-source: 99 -->";
  },
  (f: ReturnType<typeof publishedIntegrationFixture>) => {
    f.h.parents[0].sha = gate;
  },
  (f: ReturnType<typeof publishedIntegrationFixture>) => {
    f.m.parents.push({ sha: gate });
  },
  (f: ReturnType<typeof publishedIntegrationFixture>) => {
    f.m.parents[0].sha = gate;
  },
  (f: ReturnType<typeof publishedIntegrationFixture>) => {
    f.m.commit.verification.verified = false;
  },
  (f: ReturnType<typeof publishedIntegrationFixture>) => {
    f.m.commit.verification.reason = "unsigned";
  },
  (f: ReturnType<typeof publishedIntegrationFixture>) => {
    f.m.commit.tree.sha = gate;
  },
];

export function incompleteMetadataResponses() {
  const valid = metadataPage([release("v0.457.0")]);
  const page = valid.data.repository.releases;
  return [
    null,
    { ...valid, errors: [{ message: "partial response" }] },
    { data: { repository: null } },
    { data: { repository: { releases: { ...page, nodes: null } } } },
    { data: { repository: { releases: { ...page, nodes: [null] } } } },
    {
      data: {
        repository: { releases: { ...page, nodes: [{ ...page.nodes[0], isPrerelease: "false" }] } },
      },
    },
    {
      data: {
        repository: {
          releases: { ...page, nodes: [{ ...page.nodes[0], publishedAt: undefined }] },
        },
      },
    },
    metadataPage([release("v0.457.0")], true, null),
  ];
}

export function coAuthorFixtures() {
  const fix = (subject: string, ...trailers: string[]) => ({
    message: `${subject}\n\nCloses #1\n\n${trailers.join("\n")}`,
    author: "maintainer",
  });
  const claude = "Co-authored-by: Claude Opus 5.5 <noreply@anthropic.com>";
  return {
    commits: [
      fix(
        "fix(canon): a (#9768)",
        "Co-authored-by: escaton <123+escaton@users.noreply.github.com>",
        claude,
      ),
      fix(
        "fix(patina): b (#9759)",
        "Co-authored-by: naitokosuke <9+naitokosuke@users.noreply.github.com>",
      ),
      fix(
        "fix(patina): c (#9770)",
        "co-authored-by: naitokosuke <naitokosuke@users.noreply.github.com>",
      ),
      fix("fix: self (#1)", "Co-authored-by: Maintainer <1+maintainer@users.noreply.github.com>"),
      fix(
        "chore: bot (#2)",
        "Co-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
      ),
      fix("fix: mail (#3)", "Co-authored-by: Jane Doe <jane@example.com>"),
      { message: "Release v0.486.0", author: null },
    ],
    botOnly: [fix("fix: x (#4)", claude)],
  };
}

export function unrecognisedIntegrationBodies(original: string, row: string, url: string) {
  return [
    original.replace(url, `${url}0`),
    original.replace(url, `${url}?other`),
    original.replace(row, `- ${row.slice(2)}`),
    original.replace(row, `${row}\n${row}`),
    original.replace(row, `${row}\nExtra mention: ${url}`),
    original.replace(url, url.replace(repository, "foreign/repo")),
  ];
}

export function integrationLineEndingCases(original: string, row: string): [string, string][] {
  return [
    [original.replaceAll("\n", "\r\n"), `${row}\r\n`],
    [original.trimEnd(), `${row}\n`],
    [original.replace(row, `${row} with unexpected suffix`), ""],
  ];
}
