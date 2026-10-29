import type { ObjectValue } from "../../crates/vize_canon/src/binding_origins/native_api_probe/verify-codec.ts";

type Version = "7.0.2" | "7.1.0-beta";
type Role = "import" | "function";
export interface Fixture {
  requests: ObjectValue[];
  responses: ObjectValue[];
  carrier: { backendSha256: string; phases: { syntaxComplete: boolean }[] };
  clientRaw?: Buffer;
  nativeRaw?: Buffer;
}
const images: Record<Version, string> = {
  "7.0.2": "4f2de678286401759b3fb4475bafe35b8f32b4b3a07d92642bbf37eadc9b34a4",
  "7.1.0-beta": "a183b1956edbe7e7d98f2249858eedf2d8a9790809cc4909db1276e202faaa70",
};
const tags: Record<Version, Record<Role, number>> = {
  "7.0.2": { import: 277, function: 263 },
  "7.1.0-beta": { import: 278, function: 264 },
};
export const frame = (body: Buffer): Buffer =>
  Buffer.concat([Buffer.from("Content-Length: " + body.length + "\r\n\r\n", "ascii"), body]);
export const encode = (packets: ObjectValue[]): Buffer =>
  Buffer.concat(packets.map((packet) => frame(Buffer.from(JSON.stringify(packet), "utf8"))));
export function fixture(version: Version): Fixture {
  const requests: ObjectValue[] = [],
    responses: ObjectValue[] = [];
  const symbol = (name: string, id: number, role: Role = "function"): ObjectValue => {
    const declaration = id + "." + tags[version][role] + "./original/" + name + ".d.ts";
    const value: ObjectValue = {
      name,
      flags: 16,
      checkFlags: 0,
      declarations: [declaration],
      valueDeclaration: declaration,
      extra: { preserved: "🦀", array: [null, true, 12.5] },
    };
    if (version === "7.0.2") Object.assign(value, { id: String(id), project: "project" });
    else
      value.reference = {
        kind: 0,
        id,
        file: {
          fileName: "/original/probe.ts",
          path: "/original/probe.ts",
          contentHash: "a".repeat(32),
          parseOptionsKey: "0",
          scriptKind: 3,
          nodeId: String(id),
        },
      };
    return value;
  };
  const add = (method: string, params: ObjectValue, result: unknown) => {
    const id = requests.length + 1;
    requests.push({ jsonrpc: "2.0", id, method, params });
    responses.push({ jsonrpc: "2.0", id, result: structuredClone(result) });
  };
  for (let phase = 0; phase < 3; phase++) {
    const snapshot = "snapshot" + phase,
      selected = symbol("actual", 1, "import"),
      target = symbol("useTemplateRef", 2);
    const params = { snapshot, project: "project", file: "/original/probe.ts", position: 300 };
    add("getSymbolAtPosition", params, selected);
    add(
      "getAliasedSymbol",
      {
        snapshot,
        project: "project",
        symbol: version === "7.0.2" ? selected.id : selected.reference,
      },
      target,
    );
    add(
      "getSymbolAtPosition",
      { position: 300, file: "/original/probe.ts", project: "project", snapshot },
      selected,
    );
    add("getSymbolAtPosition", params, selected);
    for (let index = 0; index < 15; index++) {
      const value = index === 13 ? null : symbol("case" + index, 10 + index);
      add("getSymbolAtPosition", { ...params, position: 400 + index }, value);
      if (index < 4 && value !== null)
        add(
          "getAliasedSymbol",
          {
            snapshot,
            project: "project",
            symbol: version === "7.0.2" ? value.id : value.reference,
          },
          symbol("custom" + index, 100 + index),
        );
    }
    add("getTypeOfSymbol", { snapshot, project: "project", symbol: "annotated" }, { id: "type" });
    add("getSymbolOfType", { snapshot, project: "project", type: "type" }, target);
    add("getSourceFileNames", { snapshot, project: "project" }, ["/original/probe.ts"]);
  }
  return {
    requests,
    responses,
    carrier: {
      backendSha256: images[version],
      phases: [{ syntaxComplete: true }, { syntaxComplete: false }, { syntaxComplete: true }],
    },
  };
}
