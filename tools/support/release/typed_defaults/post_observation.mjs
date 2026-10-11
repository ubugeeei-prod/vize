import assert from "node:assert/strict";
import { apiRoot } from "./custody.mjs";

const fields = (value, keys) => Object.fromEntries(keys.map((key) => [key, value[key]]));

function assetsIdentity(assets) {
  assert(Array.isArray(assets));
  const ids = new Set();
  return assets
    .map((asset) => {
      assert(Number.isSafeInteger(asset.id) && asset.id > 0 && !ids.has(asset.id));
      ids.add(asset.id);
      assert.equal(asset.url, `${apiRoot}/releases/assets/${asset.id}`);
      return fields(asset, [
        "id",
        "name",
        "size",
        "digest",
        "state",
        "url",
        "browser_download_url",
        "content_type",
      ]);
    })
    .sort((a, b) => a.id - b.id);
}

function fixIdentity({ fix, ancestry }) {
  return {
    fix: fields(fix, ["id", "number", "merged", "merge_commit_sha"]),
    repositories: [fix.base.repo.full_name, fix.head.repo.full_name],
    ancestry: {
      base: ancestry.base_commit.sha,
      mergeBase: ancestry.merge_base_commit.sha,
      status: ancestry.status,
    },
  };
}

export function assertStableFix(before, after) {
  assert.deepEqual(fixIdentity(after), fixIdentity(before), "actual released fix identity changed");
}

function officialIdentity(official) {
  return {
    tagObject: fields(official.tagObject, ["sha", "type"]),
    tag: { ...fields(official.tag, ["sha", "tag", "message"]), target: official.tag.object },
    source: {
      ...fields(official.source, ["id", "number", "merged", "merged_at"]),
      head: fields(official.source.head, ["sha", "ref"]),
      base: official.source.base.ref,
      repositories: [official.source.base.repo.full_name, official.source.head.repo.full_name],
    },
    integrationPr: official.integrationPr,
    run: {
      ...fields(official.releaseRun, [
        "id",
        "head_sha",
        "head_branch",
        "event",
        "display_title",
        "path",
        "run_attempt",
        "status",
        "conclusion",
      ]),
      repositories: [
        official.releaseRun.repository.full_name,
        official.releaseRun.head_repository.full_name,
      ],
    },
    jobs: official.jobs,
    release: fields(official.release, [
      "id",
      "tag_name",
      "draft",
      "prerelease",
      "published_at",
      "url",
      "html_url",
    ]),
    assets: assetsIdentity(official.release.assets),
    commit: {
      sha: official.commit.sha,
      tree: official.commit.tree.sha,
      parents: official.commit.parents.map(({ sha }) => sha),
    },
    typedFix: fixIdentity(official),
  };
}

export function assertStableOfficial(before, after) {
  // Raw responses remain archived. Download counts, labels, timestamps and PR
  // discussion metadata may change while the same authenticated bytes run.
  assert.deepEqual(
    officialIdentity(after),
    officialIdentity(before),
    "official release identity changed during public observation",
  );
}
