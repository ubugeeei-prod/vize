import assert from "node:assert/strict";
import { custodyFixture } from "./docs-deployment-custody.ts";

export function history() {
  // The authenticated publisher is independent of the missing migration anchor.
  const value = custodyFixture(99001);
  const { api, deployment, publisher, job, receipt, values } = value;
  const lists = new Map<string, unknown[]>([
    [
      "/actions/workflows/deploy-docs.yml/runs?branch=main&created=" +
        encodeURIComponent(">=1970-01-01T00:00:00Z"),
      [],
    ],
    [
      "/actions/workflows/deploy-docs.yml/runs?branch=main&created=" +
        encodeURIComponent(">=" + receipt.outstanding.since),
      [],
    ],
  ]);
  for (const status of ["queued", "in_progress", "waiting", "pending", "requested"])
    lists.set("/actions/workflows/deploy-docs.yml/runs?branch=main&status=" + status, []);
  values.set(
    "/deployments?environment=vize-docs-source-custody&task=vize-docs-pages-v1&per_page=100&page=1",
    [deployment],
  );
  values.set("/deployments?environment=github-pages&per_page=100&page=1", []);
  const requests: string[] = [];
  const json = api.json.bind(api);
  api.json = async <T>(path: string): Promise<T> => {
    requests.push(path);
    assert(
      !path.includes("/actions/runs/38013719817"),
      "Missing obsolete legacy metadata cannot block current custody",
    );
    return await json<T>(path);
  };
  api.list = async <T>(path: string): Promise<T[]> => {
    requests.push(path);
    assert(lists.has(path), "Only journal-bounded primary history: " + path);
    return lists.get(path) as T[];
  };
  api.manifest = async () => {
    assert.fail("Current authentic custody never re-downloads old archives");
  };
  return { ...value, lists, requests, publisher, job };
}
