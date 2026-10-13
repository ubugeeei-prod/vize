import type { Page } from "playwright";

interface Location {
  hash?: string;
  name?: string;
}
export async function push(
  page: Page,
  location: Location,
  edge?: { tab?: string; replace?: Location },
) {
  await page.evaluate(
    async ({ location, edge }) => {
      const root = document.querySelector("#app") as unknown as {
        __vue_app__: {
          config: { globalProperties: { $router: { push(value: unknown): Promise<unknown> } } };
        };
      };
      const router = root.__vue_app__.config.globalProperties.$router;
      const pending = router.push(location);
      if (edge?.tab) {
        const button = [...document.querySelectorAll<HTMLButtonElement>(".tab-btn")].find(
          (button) => button.textContent?.trim() === edge.tab,
        );
        if (!button) throw new Error(`Missing actual tab ${edge.tab}`);
        button.click();
      }
      if (edge?.replace) await router.push(edge.replace);
      await pending;
    },
    { location, edge },
  );
}
