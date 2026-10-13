import assert from "node:assert/strict";
import type { Page } from "playwright";

const ownedSelectors = [
  ".entry-page .features::before",
  ".entry-page .hero-action-brand",
  ".entry-page .hero-action-alt",
  ".entry-page .hero-text",
  ".blog-card-kicker",
  ".blog-meta-label",
];
const themeSelectors = [
  ".nav-title",
  ".toc-title",
  ".mobile-footer-label",
  ".search-result-scope",
  ".content .ox-callout-title",
  ".content .ox-api-entry__section h4",
  ".content .ox-api-entry__member-group > h5",
  ".content .ox-api-entry__member-detail-section h6",
];

/** Inspect the loaded build's native CSSOM and actual authored elements. */
export async function verifyAuthoredLetterCase(page: Page) {
  const observed = await page.evaluate(
    ({ owned, theme }) => {
      const selectors = [...owned, ...theme];
      const rules: { selector: string; transform: string }[] = [];
      const visit = (list: CSSRuleList) => {
        for (const rule of list) {
          const styled = rule as CSSStyleRule;
          if (styled.selectorText) {
            for (const selector of styled.selectorText.split(",").map((value) => value.trim())) {
              if (selectors.includes(selector))
                rules.push({
                  selector,
                  transform: styled.style.getPropertyValue("text-transform"),
                });
            }
          }
          const group = rule as CSSGroupingRule;
          if (group.cssRules) visit(group.cssRules);
          const imported = rule as CSSImportRule;
          if (imported.styleSheet) visit(imported.styleSheet.cssRules);
        }
      };
      for (const sheet of document.styleSheets) {
        try {
          visit(sheet.cssRules);
        } catch (error) {
          // Cross-origin font styles cannot expose CSSOM. Every owned rule
          // and theme override must still be present in the readable sheets.
          if (!(error instanceof DOMException) || error.name !== "SecurityError") throw error;
        }
      }
      const elements = selectors.flatMap((selector) => {
        const pseudo = selector.endsWith("::before") ? "::before" : undefined;
        return [...document.querySelectorAll(pseudo ? selector.slice(0, -8) : selector)].map(
          (element) => {
            const style = getComputedStyle(element, pseudo);
            return {
              selector,
              authoredText: pseudo ? style.content : element.textContent?.trim(),
              transform: style.textTransform,
            };
          },
        );
      });
      return { rules, elements };
    },
    { owned: ownedSelectors, theme: themeSelectors },
  );
  for (const selector of ownedSelectors) {
    const rules = observed.rules.filter((rule) => rule.selector === selector);
    assert.ok(rules.length, `${page.url()}: loaded owned selector ${selector}`);
    assert.ok(
      rules.every((rule) => rule.transform !== "uppercase"),
      `${page.url()}: owned ${selector} forces uppercase`,
    );
  }
  for (const selector of themeSelectors) {
    assert.ok(
      observed.rules.some((rule) => rule.selector === selector && rule.transform === "none"),
      `${page.url()}: compiled theme omits natural-case override ${selector}; actual elements ${JSON.stringify(observed.elements.filter((element) => element.selector === selector))}`,
    );
  }
  for (const element of observed.elements) {
    assert.equal(
      element.transform,
      "none",
      `${page.url()}: ${element.selector} changes authored ${JSON.stringify(element.authoredText)}`,
    );
  }
  return observed;
}
