// Request pattern of one timeline-proxy log: when requests arrive and how long replies take.
export function summarize(text: string): string[] {
  const ev = text.trim().split("\n").map((l) => JSON.parse(l));
  const pending = new Map<string, number>(); const lat: number[] = []; let out = 0, maxOut = 0;
  const first = (m: string) => ev.find((e) => e.ev === "req" && e.method === m)?.t as number | undefined;
  const tr = ev.filter((e) => e.ev === "req" && e.method === "transform");
  let lastRes = 0;
  for (const e of ev) {
    const key = JSON.stringify(e.id);
    if (e.ev === "req" && e.method === "transform") { pending.set(key, e.t); out++; maxOut = Math.max(maxOut, out); }
    if (e.ev === "res" && pending.has(key)) { lat.push(e.t - pending.get(key)!); pending.delete(key); out--; lastRes = e.t; }
  }
  const s = [...lat].sort((a, b) => a - b); const q = (p: number) => s[Math.min(s.length - 1, Math.floor(p * s.length))]?.toFixed(0);
  const ms = (x: number | undefined) => (x == null ? "–" : x.toFixed(0));
  return [
    `initialize ${ms(first("initialize"))}, openProject ${ms(first("openProject"))}, transforms ${tr.length} requested ${ms(tr[0]?.t)}–${ms(tr.at(-1)?.t)}, last reply ${ms(lastRes)}`,
    `max outstanding ${maxOut}, reply latency p50 ${q(0.5)} p90 ${q(0.9)} max ${s.length ? s.at(-1)!.toFixed(0) : "–"}`,
  ];
}
if (import.meta.main) console.log(summarize((await import("node:fs")).readFileSync(process.argv[2]!, "utf8")).join("\n"));
