import { appendFileSync, readFileSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";

interface PullRequest {
  number: number;
  author_association: string;
  head: { sha: string; repo?: { full_name?: string } | null };
}
interface Snapshot {
  head: { sha: string };
  changed_files: number;
}
interface ChangedFile {
  filename: string;
  previous_filename?: string;
}
interface FilesApi {
  snapshot(): Promise<Snapshot>;
  files(): Promise<ChangedFile[]>;
}

const maintainers = new Set(["OWNER", "MEMBER", "COLLABORATOR"]);
const fileLimit = 3000;

export async function auditRequired(
  event: string | undefined,
  repository: string | undefined,
  pr: PullRequest | undefined,
  api: FilesApi,
  warn: (message: string) => void,
): Promise<boolean> {
  try {
    if (
      event !== "pull_request" ||
      !pr ||
      !repository ||
      !maintainers.has(pr.author_association) ||
      pr.head.repo?.full_name !== repository
    )
      return true;
    const before = await api.snapshot();
    if (
      before.head.sha !== pr.head.sha ||
      !Number.isInteger(before.changed_files) ||
      before.changed_files <= 0 ||
      before.changed_files >= fileLimit
    )
      return true;
    const files = await api.files();
    const after = await api.snapshot();
    return (
      before.head.sha !== after.head.sha ||
      before.changed_files !== after.changed_files ||
      files.length !== before.changed_files ||
      files.some(({ filename, previous_filename }) =>
        [filename, previous_filename].some((name) => name?.startsWith(".github/")),
      )
    );
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    warn("Unable to prove the audit is unnecessary: " + message);
    return true;
  }
}

async function main(): Promise<void> {
  const env = process.env;
  const event = JSON.parse(readFileSync(env.GITHUB_EVENT_PATH!, "utf8")) as {
    pull_request?: PullRequest;
  };
  const endpoint = `${env.GITHUB_API_URL}/repos/${env.GITHUB_REPOSITORY}/pulls/${event.pull_request?.number}`;
  async function get<T>(suffix = ""): Promise<T> {
    const response = await fetch(endpoint + suffix, {
      headers: { Accept: "application/vnd.github+json", Authorization: `Bearer ${env.GH_TOKEN}` },
    });
    if (!response.ok) throw new Error(`GitHub returned ${response.status}`);
    return (await response.json()) as T;
  }
  const audit = await auditRequired(
    env.GITHUB_EVENT_NAME,
    env.GITHUB_REPOSITORY,
    event.pull_request,
    {
      snapshot: () => get<Snapshot>(),
      async files() {
        const files: ChangedFile[] = [];
        for (let page = 1; files.length < fileLimit; page++) {
          const batch = await get<ChangedFile[]>(`/files?per_page=100&page=${page}`);
          files.push(...batch);
          if (batch.length < 100) break;
        }
        return files;
      },
    },
    (message) => console.warn(message),
  );
  appendFileSync(env.GITHUB_OUTPUT!, `audit=${audit}\n`);
  appendFileSync(env.GITHUB_STEP_SUMMARY!, `Workflow security audit required: ${audit}\n`);
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) await main();
