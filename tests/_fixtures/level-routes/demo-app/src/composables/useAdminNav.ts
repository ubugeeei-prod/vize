import { router } from "../router";

export function openAuditEntry(entryId: string) {
  return router.push({ name: "admin-audit", params: { entryId, tab: "history" } });
}

export function openUser(userId: number) {
  return router.push({ name: "user", params: { userId } });
}
