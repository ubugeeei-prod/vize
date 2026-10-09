// Run the native renderer and client preservation laws in hosted tooling discovery.
import "../../docs/theme/syntax-highlight.test.js";
import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { test } from "node:test";
import { Window } from "happy-dom";
import { annotateExampleChanges } from "../../docs/scripts/rules/inline-code-diff.ts";

void test("Vue example changes annotate changed lines while preserving complete copyable source", () => {
  const require = createRequire(new URL("../../docs/package.json", import.meta.url));
  const native = createRequire(require.resolve("@ox-content/vite-plugin"))("@ox-content/napi") as {
    transform(
      source: string,
      options: { codeAnnotations: boolean },
    ): { html: string; errors: unknown[] };
  };
  const bad = "<template>\n  <p>{{ missing }}</p>\n</template>";
  const good = "<template>\n  <p>{{ message }}</p>\n</template>";
  const reference = `## Bad\n\n\`\`\`vue\n${bad}\n\`\`\`\n\n## Good\n\n\`\`\`vue\n${good}\n\`\`\``;
  const rendered = native.transform(annotateExampleChanges(reference, bad, good), {
    codeAnnotations: true,
  });
  assert.deepEqual(rendered.errors, []);
  const window = new Window();
  try {
    window.document.body.innerHTML = rendered.html;
    const blocks = [...window.document.querySelectorAll("pre code")];
    assert.deepEqual(
      blocks.map((code) => code.textContent),
      [bad + "\n", good + "\n"],
    );
    assert.equal(
      window.document.querySelectorAll(".ox-code-line--remove[data-line='2']").length,
      1,
    );
    assert.equal(window.document.querySelectorAll(".ox-code-line--add[data-line='2']").length, 1);
    assert.equal(window.document.querySelectorAll(".diff").length, 2);
    assert.doesNotMatch(blocks[0].textContent, /^[-+]/m);
  } finally {
    window.close();
  }
});
