// Scheduled release cadence: release the newest main commit whose push Check
// passed when it adds a user-facing change since the last release tag.

const USER_FACING = /^(fix|feat|perf)(\([^)]*\))?!?:/;

export interface CadenceInput {
  /** Newest main commit whose push `Check` run succeeded, if any. */
  greenSha: string | null;
  /** main commit the latest release tag was cut from. */
  releasedSha: string | null;
  /** Subjects of the commits in releasedSha..greenSha. */
  subjects: readonly string[];
  /** A candidate for greenSha exists: an earlier run failed before tagging. */
  attempted?: boolean;
}

export function cadenceDecision({ greenSha, releasedSha, subjects, attempted }: CadenceInput) {
  if (greenSha === null) return { release: false, reason: "no green Check on main" };
  if (greenSha === releasedSha) return { release: false, reason: "already released" };
  // Retry only from a newer green commit; dispatch retries the same one.
  if (attempted) return { release: false, reason: "a release of this commit already failed" };
  const changes = subjects.filter((subject) => USER_FACING.test(subject));
  return changes.length > 0
    ? { release: true, reason: `${changes.length} fix/feat/perf commits since the last release` }
    : { release: false, reason: "no fix/feat/perf commits since the last release" };
}

type Json = Record<string, unknown>;

async function api(path: string): Promise<Json> {
  const response = await fetch(
    `https://api.github.com/repos/${process.env.GITHUB_REPOSITORY}/${path}`,
    {
      headers: {
        accept: "application/vnd.github+json",
        authorization: `Bearer ${process.env.GITHUB_TOKEN}`,
      },
    },
  );
  if (!response.ok) throw new Error(`GET ${path}: HTTP ${response.status}`);
  return (await response.json()) as Json;
}

/** Reads the cadence inputs from the GitHub API. `lastTag` is the highest release tag. */
export async function readCadenceInput(lastTag: string | null): Promise<CadenceInput> {
  const runs = await api(
    "actions/workflows/check.yml/runs?branch=main&event=push&status=success&per_page=1",
  );
  const run = (runs.workflow_runs as Json[] | undefined)?.[0];
  const greenSha = typeof run?.head_sha === "string" ? run.head_sha : null;
  if (greenSha === null || lastTag === null) return { greenSha, releasedSha: null, subjects: [] };
  // A release tag commit's only parent is the main commit it was cut from.
  const tagCommit = await api(`commits/${encodeURIComponent(lastTag)}`);
  const releasedSha = ((tagCommit.parents as Json[] | undefined)?.[0]?.sha as string) ?? null;
  if (releasedSha === null || releasedSha === greenSha)
    return { greenSha, releasedSha, subjects: [] };
  const compare = await api(`compare/${releasedSha}...${greenSha}`);
  const subjects = ((compare.commits as Json[] | undefined) ?? []).map((commit) => {
    const message = (commit.commit as Json | undefined)?.message;
    return typeof message === "string" ? message.split("\n")[0] : "";
  });
  return { greenSha, releasedSha, subjects };
}
