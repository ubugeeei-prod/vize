import { createRequire } from "node:module";
import type { TsserverClient } from "./tsserver-client.ts";

const probe = createRequire(import.meta.url)("./classification-probe.cjs") as {
  createClassificationFixtures(workspace: string, output: string): ClassificationFixture;
  checkClassifications(fixture: ClassificationFixture, adapter: object): Promise<object>;
  semantic: string;
  syntactic: string;
};
export interface ClassificationFixture {
  readonly output: string;
}
export const createClassificationFixtures = probe.createClassificationFixtures;

export async function checkPackedClassifications(
  client: TsserverClient,
  fixture: ClassificationFixture,
): Promise<object> {
  return probe.checkClassifications(fixture, {
    transport: "tsserver",
    commands: [probe.syntactic, probe.semantic],
    open: (file: string, text: string) =>
      client.send("open", { file, fileContent: text, scriptKindName: "TS" }, false),
    change: (file: string, before: string, after: string) => {
      const lines = before.split("\n");
      return client.send("change", {
        file,
        line: 1,
        offset: 1,
        endLine: lines.length,
        endOffset: lines.at(-1)!.length + 1,
        insertString: after,
      });
    },
    query: (command: string, file: string, _text: string, range: object) =>
      client.send(command, { file, ...range, format: "2020" }),
  });
}
