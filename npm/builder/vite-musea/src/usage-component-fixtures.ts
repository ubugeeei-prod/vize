import { readFile } from "node:fs/promises";

export interface UsageComponentCase {
  name: string;
  filename: string;
  title: string;
  scriptSetup?: string;
  componentAttribute?: string;
  componentTagName: string;
  propsInterface: string;
  inline: boolean;
}

export async function usageComponentFixture() {
  return JSON.parse(
    await readFile(
      new URL(
        "../../../../tests/_fixtures/differential/musea/usage-component-tag.json",
        import.meta.url,
      ),
      "utf8",
    ),
  ) as {
    componentSource: string;
    editedLabel: string;
    editedConstructor: string;
    editedHasOwnProperty: string;
    customName: string;
    customValue: string;
    reservedValue: Record<string, unknown>;
    cases: UsageComponentCase[];
  };
}

export function usageArtSource(fixture: UsageComponentCase, componentSource: string): string {
  const script = fixture.inline
    ? componentSource
    : fixture.scriptSetup
      ? `<script setup lang="ts">\n${fixture.scriptSetup}\n</script>\n`
      : "";
  const component = fixture.componentAttribute ? ` component="${fixture.componentAttribute}"` : "";
  return `${script}\n<art title="${fixture.title}"${component}><variant name="Default" default><Self label="Seed" constructor="Seed constructor" hasOwnProperty="Seed method" /></variant></art>`;
}
