import assert from "node:assert/strict";

export type CapacityProbe = {
  file: string;
  uri: string;
  invalid: string;
  valid: string;
  token: string;
  line: number;
  character: number;
};
export type CapacityPacket = {
  uri: string;
  version?: number;
  diagnostics: {
    code?: unknown;
    source?: string;
    message?: string;
    range?: {
      start?: { line?: number; character?: number };
      end?: { line?: number; character?: number };
    };
  }[];
};

export function capacitySource(index: number, uri: string): CapacityProbe {
  const id = String(index).padStart(4, "0");
  const token = "missingNative" + id;
  const eol = index % 2 ? "\n" : "\r\n";
  const valid = [
    '<script setup lang="ts">',
    "defineProps<{ file" + id + "?: string }>()",
    "const value" + id + " = 0",
    "const items = [1, 2, 3]",
    "/*🌱日本語*/ value" + id + ".toFixed()",
    "</script>",
    '<template><ul class="probe"><li v-for="item in items" :key="item">{{ item }} {{ value' +
      id +
      " }} {{ file" +
      id +
      " }}</li></ul></template>",
    "<style scoped>.probe { color: #123456 }</style>",
    "",
  ].join(eol);
  const invalid = valid.replace(".toFixed()", "." + token + "()").replace(' :key="item"', "");
  const before = invalid.slice(0, invalid.indexOf(token));
  const rows = before.split("\n");
  return {
    file: "Coverage" + id + ".vue",
    uri,
    invalid,
    valid,
    token,
    line: rows.length - 1,
    character: (rows[rows.length - 1] ?? "").length,
  };
}

export function capacityPacket(value: unknown): value is CapacityPacket {
  if (
    typeof value !== "object" ||
    value === null ||
    !("uri" in value) ||
    typeof value.uri !== "string" ||
    !("diagnostics" in value) ||
    !Array.isArray(value.diagnostics)
  )
    return false;
  return value.diagnostics.every((row: unknown) => typeof row === "object" && row !== null);
}

type IndexedPublication = {
  count: number;
  latestNative: boolean;
  latestEmpty: boolean;
  latestPacketIndex: number;
  matchingNative: number;
  matchingEmpty: number;
};

function exactNative(probe: CapacityProbe, packet: CapacityPacket): boolean {
  const native = packet.diagnostics.filter((row) => row.source === "vize/types");
  const other = packet.diagnostics.filter((row) => row.source !== "vize/types");
  return (
    native.length === 1 &&
    other.length <= 1 &&
    other.every((row) => row.source === "vize/lint" && row.code === "vue/require-v-for-key") &&
    native.every(
      (row) =>
        Number(row.code) === 2339 &&
        typeof row.message === "string" &&
        row.message.includes(probe.token) &&
        row.range?.start?.line === probe.line &&
        row.range?.start?.character === probe.character &&
        row.range?.end?.line === probe.line &&
        row.range?.end?.character === probe.character + probe.token.length,
    )
  );
}

/** Raw multiplicity is retained; only the latest versioned publication defines current state. */
export class CapacityCoverage {
  readonly packets: { atMs: number; packet: CapacityPacket }[] = [];
  readonly probes: CapacityProbe[];
  private readonly byUri = new Map<string, CapacityProbe>();
  private readonly index = new Map<string, Map<number, IndexedPublication>>();
  private readonly syncIndex = new Map<string, { packets: number; accepted: boolean }>();
  constructor(probes: CapacityProbe[]) {
    this.probes = probes;
    for (const probe of probes) this.byUri.set(probe.uri, probe);
    assert.equal(this.byUri.size, probes.length, "every probe must own a unique URI");
  }
  record(packet: CapacityPacket, atMs: number): void {
    const latestPacketIndex = this.packets.length;
    this.packets.push({ atMs, packet });
    const probe = this.byUri.get(packet.uri);
    if (!probe) return;
    if (packet.version === undefined) {
      const old = this.syncIndex.get(packet.uri);
      this.syncIndex.set(packet.uri, {
        packets: (old?.packets ?? 0) + 1,
        accepted:
          Boolean(old?.accepted) ||
          packet.diagnostics.some(
            (row) => row.source === "vize/lint" && row.code === "vue/require-v-for-key",
          ),
      });
      return;
    }
    const versions = this.index.get(packet.uri) ?? new Map<number, IndexedPublication>();
    const old = versions.get(packet.version);
    const latestNative = exactNative(probe, packet);
    const latestEmpty = packet.diagnostics.length === 0;
    versions.set(packet.version, {
      count: (old?.count ?? 0) + 1,
      latestNative,
      latestEmpty,
      latestPacketIndex,
      matchingNative: (old?.matchingNative ?? 0) + Number(latestNative),
      matchingEmpty: (old?.matchingEmpty ?? 0) + Number(latestEmpty),
    });
    this.index.set(packet.uri, versions);
  }
  sync(uri: string) {
    return this.syncIndex.get(uri) ?? { packets: 0, accepted: false };
  }
  rows(version: number, repaired: boolean) {
    return this.probes.map((probe) => {
      const state = this.index.get(probe.uri)?.get(version);
      return {
        file: probe.file,
        uri: probe.uri,
        token: probe.token,
        expected: { line: probe.line, character: probe.character, code: 2339 },
        publications: state?.count ?? 0,
        historicalMatchingPackets: (repaired ? state?.matchingEmpty : state?.matchingNative) ?? 0,
        latestPacketIndex: state?.latestPacketIndex ?? null,
        complete: state !== undefined && (repaired ? state.latestEmpty : state.latestNative),
      };
    });
  }
  assertComplete(version: number, repaired: boolean): void {
    const rows = this.rows(version, repaired);
    assert.equal(rows.length, this.probes.length);
    assert.deepEqual(
      rows.filter((row) => !row.complete).map((row) => row.file),
      [],
      "every independent open document must retain its complete current native result",
    );
  }
}

export const waitUntil = async (
  condition: () => boolean,
  milliseconds: number,
): Promise<boolean> => {
  const deadline = Date.now() + milliseconds;
  while (!condition() && Date.now() < deadline)
    await new Promise((resolve) => setTimeout(resolve, 50));
  return condition();
};
