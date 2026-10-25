export const memberCounts = [32, 128, 512];
export const memberShapes = [
  "ascii",
  "readonly",
  "quoted",
  "readonlyQuoted",
  "unicode",
  "readonlyUnicode",
];
export const profileKeys = [
  "cacheHit",
  "descriptorParseCalls",
  "analyzeCalls",
  "literalMemberParseCalls",
  "defaultParseCalls",
  "serializeCalls",
  "descriptorParseNs",
  "analyzeNs",
  "defaultProjectionNs",
  "serializeNs",
].sort();
export function memberCases() {
  return memberCounts.flatMap((count) =>
    memberShapes.map((shape) => {
      const props = Array.from({ length: count }, (_, i) => ({
        name: shape.toLowerCase().includes("unicode") ? `café_${i}` : `p_${i}`,
        type: "number",
        required: false,
      }));
      const members = props
        .map(
          (prop) =>
            `${shape.startsWith("readonly") ? "readonly " : ""}${shape.toLowerCase().includes("quoted") ? JSON.stringify(prop.name) : prop.name}?: number;`,
        )
        .join(" ");
      return {
        id: `${shape}-${count}`,
        count,
        shape,
        source: `<script setup lang="ts">defineProps<{ ${members} }>()</script>`,
        expected: { emits: [], props },
      };
    }),
  );
}
export function controlCases() {
  return [
    {
      id: "keyword",
      members: "readonly?: string; readonlyThing?: number; readonly readonlyName?: boolean",
      props: [
        { name: "readonly", type: "string", required: false },
        { name: "readonlyThing", type: "number", required: false },
        { name: "readonlyName", type: "boolean", required: false },
      ],
    },
    {
      id: "whitespace",
      members:
        "readonly\tp0?: number; readonly/* boundary */p1?: number; readonly\u00a0p2?: number",
      props: ["p0", "p1", "p2"].map((name) => ({ name, type: "number", required: false })),
    },
    {
      id: "cooked-refused",
      members: String.raw`readonly "scope.name"?: string; readonly ["a:b"]?: number; "\u006cabel"?: string; [key]: string; [sideEffect()]: number`,
      props: [
        { name: "scope.name", type: "string", required: false },
        { name: "a:b", type: "number", required: false },
        { name: "label", type: "string", required: false },
      ],
    },
    {
      id: "readonly-optional-spacing",
      members:
        "readonly p0? : number; readonly p1?\t: number; readonly/* modifier */p2? /* optional */ : number",
      props: ["p0", "p1", "p2"].map((name) => ({ name, type: "number", required: false })),
    },
  ].map(({ id, members, props }) => ({
    id,
    source: `<script setup lang="ts">defineProps<{ ${members} }>()</script>`,
    expected: { emits: [], props },
  }));
}
export function percentiles(values) {
  const sorted = values.toSorted((a, b) => a - b);
  if (!sorted.length || sorted.some((value) => !Number.isFinite(value) || value < 0))
    throw Error("invalid timing vector");
  return {
    p50: sorted[Math.ceil(sorted.length * 0.5) - 1],
    p95: sorted[Math.ceil(sorted.length * 0.95) - 1],
    min: sorted[0],
    max: sorted.at(-1),
  };
}
