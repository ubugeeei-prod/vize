import assert from "node:assert/strict";
import { createRequire } from "node:module";

const fromUi = createRequire(new URL("../../../npm/ui/package.json", import.meta.url));
const compiler = fromUi("vue/compiler-sfc");
const vue = fromUi("vue");

const serialize = (error) =>
  typeof error === "string"
    ? error
    : { code: error.code ?? null, message: error.message, loc: error.loc ?? null };

export function diagnostics(source, fixture) {
  const parsed = compiler.parse(source, { filename: fixture.filename });
  const id = "data-v-rule-layout";
  const styles = parsed.descriptor.styles.map((style) => {
    const compiled = compiler.compileStyle({
      source: style.content,
      filename: fixture.filename,
      id,
      scoped: style.scoped,
    });
    return { code: compiled.code, errors: compiled.errors.map(serialize) };
  });
  let dom = null;
  let ssr = null;
  if (parsed.descriptor.template) {
    const options = {
      source: parsed.descriptor.template.content,
      filename: fixture.filename,
      id,
      cssVars: parsed.descriptor.cssVars,
    };
    const compile = (ssr) => {
      const result = compiler.compileTemplate({ ...options, ssr });
      return { code: result.code, errors: result.errors.map(serialize), tips: result.tips };
    };
    dom = compile(false);
    ssr = compile(true);
  }
  const result = { vue: vue.version, parse: parsed.errors.map(serialize), styles, dom, ssr };
  assert.deepEqual(
    result.parse,
    fixture.expectedParseErrors,
    `${fixture.id}: whole original parse vector`,
  );
  for (const style of styles) assert.deepEqual(style.errors, [], `${fixture.id}: style compile`);
  if (dom) assert.deepEqual(dom.errors, [], `${fixture.id}: whole DOM compile`);
  if (ssr) assert.deepEqual(ssr.errors, [], `${fixture.id}: whole SSR compile`);
  return result;
}

export async function observeCss(browser, compiled) {
  const page = await browser.newPage();
  try {
    return await page.evaluate(
      (styles) => {
        const sheet = new CSSStyleSheet();
        sheet.replaceSync(styles.join("\n"));
        document.adoptedStyleSheets = [sheet];
        document.body.innerHTML =
          '<div class="a b c 次" data-label="a,b" data-v-rule-layout></div>';
        const computed = getComputedStyle(document.body.firstElementChild);
        return {
          cssRules: Array.from(sheet.cssRules, (rule) => rule.cssText),
          html: document.body.innerHTML,
          computed: Array.from(computed)
            .sort()
            .map((name) => [name, computed.getPropertyValue(name)]),
        };
      },
      compiled.styles.map((style) => style.code),
    );
  } finally {
    await page.close();
  }
}
