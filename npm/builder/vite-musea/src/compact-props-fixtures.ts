import { readFile } from "node:fs/promises";

export interface CompactPropsFixture {
  originalCompactSourceSha256: string;
  originalMultilineSourceSha256: string;
  cases: Array<{
    name: string;
    source: string;
    expected: {
      props: Array<{ name: string; type: string; required: boolean; default_value?: string }>;
      emits: string[];
    };
  }>;
  browserCases: Array<{
    name: string;
    filename: string;
    title: string;
    source: string;
    names: string[];
  }>;
  editedValues: Record<string, string>;
}

export async function compactPropsFixture(): Promise<CompactPropsFixture> {
  return JSON.parse(
    await readFile(
      new URL("../../../../tests/_fixtures/differential/musea/compact-props.json", import.meta.url),
      "utf8",
    ),
  );
}

export function compactPropsArtSource(vector: CompactPropsFixture["browserCases"][number]): string {
  const literal = vector.names.includes("scope.name")
    ? `v-bind="{['scope.name']:'Seed custom',['__proto__']:'Seed proto'}"`
    : "";
  return `<script setup lang="ts">
import { default as CompactProbe } from './${vector.filename}';
defineArt(CompactProbe, { title: '${vector.title}' });
</script>
<art><variant name="Default"><Self label="Seed" constructor="Seed constructor" hasOwnProperty="Seed method" ${literal} /></variant></art>`;
}
