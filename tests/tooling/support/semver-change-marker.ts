export type MergedPullRequest = {
  body: string;
  merge_commit_sha: string;
  merged_at: string;
  number: number;
  title: string;
};

export const squashSha = "f49c94e03ad957b1f6f51276a328acb533c21343";
export const previousSha = "e39ff7493a5f9b9d0e6d3a05ff3d8fbfc0f1b2a3";

// Modelled on #3720: the accepted breaking exception lives in the pull-request
// body, which is exactly the part a squash merge does not carry over.
export const breakingPullRequest: MergedPullRequest = {
  body: [
    "## Compatibility note",
    "",
    "No released API is removed: the field existed only on transient `main`.",
    "",
    "BREAKING CHANGE: remove the unreleased `ElementNode::is_custom_element` field.",
    "",
  ].join("\n"),
  merge_commit_sha: squashSha,
  merged_at: "2026-08-02T08:40:34Z",
  number: 3720,
  title: "fix(atelier): keep custom element metadata internal",
};

/**
 * Rebuild the squash commit message the way GitHub does: the pull-request
 * title, its number, and the squashed commits' own messages. The pull-request
 * body never takes part, so any marker that only lived there is destroyed.
 */
export function squashCommitMessage(
  pullRequest: MergedPullRequest,
  commitMessages: string[],
): string {
  const subject = `${pullRequest.title} (#${pullRequest.number})`;
  const lines = commitMessages.length === 0 ? [subject] : [subject, "", ...commitMessages];
  return `${lines.join("\n")}\n`;
}
