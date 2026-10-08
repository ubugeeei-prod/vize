import assert from "node:assert/strict";
import { test } from "node:test";
import { formatSfc } from "../../npm/native/index.js";

test("native formatter selects Vue 2 filters explicitly and defaults to Vue 3", () => {
  const source =
    "<template><p :title=\"message | format-date('en')\">{{message|format-date('en')|suffix('!')}}</p></template>\n";
  const vue2 =
    '<template>\n  <p :title="message | format-date(\'en\')">{{ message | format-date("en") | suffix("!") }}</p>\n</template>\n';
  const vue3 =
    '<template>\n  <p :title="message | (format - date(\'en\'))">{{ message | (format - date("en")) | suffix("!") }}</p>\n</template>\n';
  for (const vueVersion of [undefined, "3", "2", "2.7"]) {
    const options = vueVersion === undefined ? {} : { vueVersion };
    const expected = vueVersion === "2" || vueVersion === "2.7" ? vue2 : vue3;
    const first = formatSfc(source, options);
    assert.deepEqual(first, { code: expected, changed: true });
    for (let pass = 2; pass <= 3; pass += 1) {
      assert.deepEqual(formatSfc(first.code, options), { code: expected, changed: false });
    }
  }
  for (const vueVersion of ["unknown", "0", "2.0"]) {
    assert.throws(() => formatSfc(source, { vueVersion }), /vue.version/);
  }
});
