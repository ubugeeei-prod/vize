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
      " }}</li></ul></template>",
    "<style>.probe { color: #123456 }</style>",
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

/** Keep complete packets and duplicate publications; an empty/pending response is not an error witness. */
export class CapacityCoverage {
  readonly packets: { atMs: number; packet: CapacityPacket }[] = [];
  readonly probes: CapacityProbe[];
  constructor(probes: CapacityProbe[]) {
    this.probes = probes;
  }
  record(packet: CapacityPacket, atMs: number): void {
    this.packets.push({ atMs, packet });
  }
  rows(version: number, repaired: boolean) {
    return this.probes.map((probe) => {
      const publications = this.packets.filter(
        ({ packet }) => packet.uri === probe.uri && packet.version === version,
      );
      const complete = publications.filter(({ packet }) =>
        repaired
          ? packet.diagnostics.length === 0
          : packet.diagnostics.filter((row) => row.source === "vize/types").length === 1 &&
            packet.diagnostics
              .filter((row) => row.source !== "vize/types")
              .every((row) => row.source === "vize/lint" && row.code === "vue/require-v-for-key") &&
            packet.diagnostics
              .filter((row) => row.source === "vize/types")
              .every(
                (row) =>
                  Number(row.code) === 2339 &&
                  row.source === "vize/types" &&
                  typeof row.message === "string" &&
                  row.message.includes(probe.token) &&
                  row.range?.start?.line === probe.line &&
                  row.range?.start?.character === probe.character &&
                  row.range?.end?.line === probe.line &&
                  row.range?.end?.character === probe.character + probe.token.length,
              ),
      );
      return {
        file: probe.file,
        uri: probe.uri,
        token: probe.token,
        expected: { line: probe.line, character: probe.character, code: 2339 },
        publications: publications.length,
        matchingPackets: complete.length,
        complete: complete.length > 0,
      };
    });
  }
  assertComplete(version: number, repaired: boolean): void {
    const rows = this.rows(version, repaired);
    assert.equal(rows.length, this.probes.length);
    assert.deepEqual(
      rows.filter((row) => !row.complete).map((row) => row.file),
      [],
      "every independent open document must receive its complete current native result",
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
