import assert from "node:assert/strict";
import test from "node:test";

import { prepareWorkaroundSource } from "../workaround.ts";
import { rewriteReportedLocations } from "./locations.ts";

await test("full JSON keeps all foreign fields and maps every owned mirror/copy label", () => {
  const source = '<template>雪😀\r\n<div v-html="x" /></template>';
  const bridge = prepareWorkaroundSource(source, "/repo/Static.vue");
  const from = source.indexOf("v-html");
  const before = source.slice(0, from);
  const offset = Buffer.byteLength(before);
  const span = { offset, length: 10, line: 2, column: 6 };
  const labels = [
    { text: "mirror", span: { ...span, offset: bridge.locations.scriptStart + from } },
    { text: "copied JS", span: { ...span, offset: bridge.locations.originalStart + offset } },
    {
      span: {
        offset: bridge.locations.scriptStart + source.length,
        length: 0,
        line: 2,
        column: 33,
      },
    },
  ];
  const foreign = {
    filename: "outside.ts",
    labels: [{ span: { offset: 7, length: 1, line: 1, column: 8 } }],
  };
  const report = {
    diagnostics: [
      {
        filename: "temporary.vue",
        code: "vize(vue/no-v-html)",
        message: 'literal "span": {"offset": 9}',
        labels,
      },
      foreign,
    ],
    opaque: { value: 42 },
    number_of_files: 2,
  };
  const raw = JSON.stringify(report, null, 2) + "\n";
  const expected = {
    ...report,
    diagnostics: [
      {
        ...report.diagnostics[0],
        labels: [
          { text: "mirror", span },
          { text: "copied JS", span },
          {
            span: {
              offset: Buffer.byteLength(source),
              length: 0,
              line: 2,
              column: source.split("\n")[1].length + 1,
            },
          },
        ],
      },
      foreign,
    ],
  };
  const mapped = rewriteReportedLocations(raw, new Map([["temporary.vue", bridge.locations]]));
  assert.deepEqual(JSON.parse(mapped), expected);
  assert.equal(mapped, JSON.stringify(expected, null, 2) + "\n");
  const mirror = bridge.source.slice(
    bridge.locations.scriptStart,
    bridge.locations.scriptStart + source.length,
  );
  assert.equal(mirror.length, source.length);
  assert.equal(mirror.indexOf("\r\n"), 13);
  assert.match(mirror, /^[ \r\n]*$/u);
});

await test("plain and stylish zero-width first-line positions use the owning file only", () => {
  const bridge = prepareWorkaroundSource("<template>\n<div />\n</template>\n", "/repo/Static.vue");
  const column = bridge.locations.scriptStart + 11;
  const raw = `temporary.vue:1:${column}: whole unix message with quoted:8:9: text [Error/vize(vue/multi-word-component-names)]\ntemporary.vue:1:${column}: error vize(vue/multi-word-component-names): whole message\ntemporary.vue\n  1:${column}  error  whole message  vize(vue/multi-word-component-names)\noutside.ts\n  1:${column}  error  whole message  vize(other)\n`;
  const expected = `temporary.vue:1:11: whole unix message with quoted:8:9: text [Error/vize(vue/multi-word-component-names)]\ntemporary.vue:1:11: error vize(vue/multi-word-component-names): whole message\ntemporary.vue\n  1:11  error  whole message  vize(vue/multi-word-component-names)\noutside.ts\n  1:${column}  error  whole message  vize(other)\n`;
  assert.equal(
    rewriteReportedLocations(raw, new Map([["temporary.vue", bridge.locations]])),
    expected,
  );
});

await test("synthetic gaps and half-surrogates retain their exact unowned range", () => {
  const bridge = prepareWorkaroundSource("😀", "/repo/Static.vue");
  const report = {
    diagnostics: [
      {
        filename: "temporary.vue",
        labels: [
          { span: { offset: bridge.locations.scriptStart + 1, length: 0, line: 1, column: 1 } },
          { span: { offset: bridge.locations.scriptStart + 3, length: 0, line: 1, column: 1 } },
          { span: { offset: bridge.locations.originalStart + 3, length: 0, line: 1, column: 1 } },
        ],
      },
    ],
  };
  const raw = JSON.stringify(report);
  assert.equal(rewriteReportedLocations(raw, new Map([["temporary.vue", bridge.locations]])), raw);
});

await test("colored stylish rows preserve whole multiline messages and foreign ownership", () => {
  const source = '<template><!-- 雪😀 --><div v-html="value" /></template>\n';
  const bridge = prepareWorkaroundSource(source, "/repo/Unicode.vue");
  const color = String.fromCharCode(27);
  const filename = `${color}[4mtemporary.vue${color}[0m`;
  const warning = `${color}[33mwarning${color}[0m`;
  const error = `${color}[31merror${color}[0m`;
  const position = (column: number, padding = "") => `${color}[2m1:${column}${padding}${color}[0m`;
  const warningColumn = bridge.locations.scriptStart + 28;
  const errorColumn = bridge.locations.scriptStart + 11;
  const raw = `\n${filename}\n  ${position(warningColumn)}  ${warning}  complete first message\n    Help: quoted:8:9: coordinates stay exact\n      full next line  ${color}[2mvize(vue/no-v-html)${color}[0m\n  ${position(errorColumn, "  ")}  ${error}  complete second message\n${color}[4moutside.ts${color}[0m\n  ${position(errorColumn)}  ${error}  complete foreign message\n${color}[31mwhole summary${color}[0m\n`;
  const expected = `\n${filename}\n  ${position(28)}  ${warning}  complete first message\n    Help: quoted:8:9: coordinates stay exact\n      full next line  ${color}[2mvize(vue/no-v-html)${color}[0m\n  ${position(11, "  ")}  ${error}  complete second message\n${color}[4moutside.ts${color}[0m\n  ${position(errorColumn)}  ${error}  complete foreign message\n${color}[31mwhole summary${color}[0m\n`;
  assert.equal(
    rewriteReportedLocations(raw, new Map([["temporary.vue", bridge.locations]])),
    expected,
  );
});

await test("default graphical headers keep exact colored owners and every surrounding frame byte", () => {
  const bridge = prepareWorkaroundSource("<template>\n<div />\n</template>\n", "/repo/Static.vue");
  const column = bridge.locations.scriptStart + 11;
  const color = String.fromCharCode(27);
  const filename = `${color}[38;2;92;157;255;1mtemporary.vue${color}[0m`;
  const raw = `whole message\n   ╭─[${filename}:1:${column}]\n 1 │ complete original frame content\n   · exact marker bytes\n   ╰────\n   ╭─[outside.ts:1:${column}]\nsummary bytes\n`;
  const expected = `whole message\n   ╭─[${filename}:1:11]\n 1 │ complete original frame content\n   · exact marker bytes\n   ╰────\n   ╭─[outside.ts:1:${column}]\nsummary bytes\n`;
  assert.equal(
    rewriteReportedLocations(raw, new Map([["temporary.vue", bridge.locations]])),
    expected,
  );
});
