export function withoutHelp(packet) {
  return packet.map((result) => ({
    ...result,
    messages: result.messages.map((message) => {
      const normalized = { ...message };
      delete normalized.help;
      return normalized;
    }),
  }));
}
