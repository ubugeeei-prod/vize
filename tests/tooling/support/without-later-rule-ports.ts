// #12866 ported these rules into preset all after the packets in these corpora
// were captured. They report on the corpora's unrelated `lang="ts"` blocks and
// undocumented props, so a preset-all run is compared without their messages.
// Every other message, count and byte is still compared exactly.
const LATER_RULES = new Set([
  "vue/block-lang",
  "vue/require-default-export",
  "vue/require-prop-comment",
]);

type Message = { ruleId?: string; severity?: number };
type FileReport = { messages: Message[]; errorCount: number; warningCount: number };

export function withoutLaterRulePorts<Packet>(packet: Packet): Packet {
  const current = structuredClone(packet) as unknown as FileReport[];
  for (const report of current) {
    const kept = report.messages.filter((message) => !LATER_RULES.has(message.ruleId ?? ""));
    for (const message of report.messages) {
      if (kept.includes(message)) continue;
      if (message.severity === 2) report.errorCount -= 1;
      else report.warningCount -= 1;
    }
    report.messages = kept;
  }
  return current as unknown as Packet;
}

/** Whether a later-ported rule reported an error, which alone makes the run exit 1. */
export function hasLaterRulePortError(packet: unknown): boolean {
  return (packet as FileReport[]).some((report) =>
    report.messages.some(
      (message) => message.severity === 2 && LATER_RULES.has(message.ruleId ?? ""),
    ),
  );
}
