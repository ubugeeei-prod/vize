import {
  InstalledAliasError,
  type Packet,
  type Pending,
  type PublicationWait,
} from "./protocol-types.ts";

/** Preserve every publication; only resolve the exact original URI/version waiter. */
export function settlePublication(
  packet: Packet,
  notificationCount: number,
  publications: PublicationWait[],
  failures: string[],
): void {
  const params = packet.params as Packet | undefined;
  // Matching waits are removed below, so retain the complete original snapshot.
  for (const wait of publications.slice()) {
    if (
      notificationCount <= wait.after ||
      params?.uri !== wait.uri ||
      params?.version !== wait.version
    )
      continue;
    if (Array.isArray(params.diagnostics)) {
      wait.packets.push(packet);
      if (wait.packets.length !== wait.count) continue;
      clearTimeout(wait.timer);
      publications.splice(publications.indexOf(wait), 1);
      wait.resolve(wait.packets);
    } else {
      clearTimeout(wait.timer);
      publications.splice(publications.indexOf(wait), 1);
      failures.push("malformed diagnostic publication");
      wait.reject(new InstalledAliasError("malformed diagnostic publication", packet));
    }
  }
}

export function settleReply(
  packet: Packet,
  pendingReplies: Map<number, Pending>,
  failures: string[],
): void {
  if (typeof packet.id !== "number" || packet.method !== undefined) return;
  const pending = pendingReplies.get(packet.id);
  if (!pending) return;
  clearTimeout(pending.timer);
  pendingReplies.delete(packet.id);
  pending.outcome.response = packet;
  pending.outcome.status = "error" in packet ? "rpc-error" : "response";
  if ("error" in packet) {
    pending.outcome.error = `JSON-RPC error for ${pending.outcome.method}: ${JSON.stringify(packet.error)}`;
    if (pending.observed) pending.resolve(packet);
    else {
      failures.push(pending.outcome.error);
      pending.reject(new InstalledAliasError(pending.outcome.error, packet));
    }
  } else if (!("result" in packet)) {
    pending.outcome.status = "malformed-response";
    pending.outcome.error = "response has neither result nor error";
    failures.push(pending.outcome.error);
    pending.reject(new InstalledAliasError(pending.outcome.error, packet));
  } else pending.resolve(packet);
}
