import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import ts from "typescript";

void test("Musea variant contracts contextually type props, attrs and inline play", () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "musea-client-types-"));
  const clientTypes = path.resolve(import.meta.dirname, "../client.d.ts");
  const fixture = path.join(directory, "variant-api.ts");

  try {
    fs.writeFileSync(
      fixture,
      `/// <reference path=${JSON.stringify(clientTypes)} />
declare const Counter: new () => {
  $props: { initial: number; step?: number };
  $attrs: { "aria-label"?: string; "data-testid"?: string };
};

defineArt(Counter, { title: "Counter" });
defineArtMeta({ category: "Input", status: "ready" });
// @ts-expect-error defineArt only accepts a component value.
defineArt("./Counter.vue", { title: "Counter" });
// @ts-expect-error defineArtMeta status is a closed union.
defineArtMeta({ status: "wip" });

const props = { initial: 0, step: 1 } satisfies Partial<MuseaComponentProps<typeof Counter>>;
const attrs = { "aria-label": "Counter" } satisfies MuseaComponentAttrs<typeof Counter>;
const play: MuseaPlay = async ({ canvas, user, expect, step }) => {
  await step("increment", async () => {
    await user.click(canvas.getByRole("button", { name: "Increment" }));
    expect(canvas.getByText("Count: 1")).toBeVisible();
  });
};

void props;
void attrs;
void play;

// @ts-expect-error Counter.initial is numeric.
const invalidProps = { initial: "zero" } satisfies Partial<MuseaComponentProps<typeof Counter>>;
// @ts-expect-error Counter aria-label is textual.
const invalidAttrs = { "aria-label": 1 } satisfies MuseaComponentAttrs<typeof Counter>;
// @ts-expect-error MuseaCanvas has no unscoped document query.
const invalidPlay: MuseaPlay = ({ canvas }) => canvas.getByTestId("counter");
void invalidProps;
void invalidAttrs;
void invalidPlay;
`,
      "utf8",
    );

    const program = ts.createProgram([fixture], {
      target: ts.ScriptTarget.ES2022,
      module: ts.ModuleKind.NodeNext,
      moduleResolution: ts.ModuleResolutionKind.NodeNext,
      strict: true,
      noEmit: true,
      skipLibCheck: true,
    });
    const diagnostics = ts
      .getPreEmitDiagnostics(program)
      .filter((diagnostic) => diagnostic.file?.fileName.startsWith(directory));
    assert.deepEqual(
      diagnostics.map((diagnostic) =>
        ts.flattenDiagnosticMessageText(diagnostic.messageText, "\n"),
      ),
      [],
    );
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});
