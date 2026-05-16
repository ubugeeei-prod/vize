import { spawnSync } from "node:child_process";
import type { Page } from "@playwright/test";

export function assertParsesAsModule(source: string, fileLabel: string): void {
  const result = spawnSync(process.execPath, ["--input-type=module", "--check"], {
    encoding: "utf-8",
    input: source,
  });

  if (result.error) {
    throw result.error;
  }

  if (result.status !== 0) {
    const details =
      result.stderr.trim() ||
      result.stdout.trim() ||
      `node --check exited with status ${result.status}`;
    throw new Error(`Invalid JS in ${fileLabel}: ${details}`);
  }
}

export async function collectConsoleErrors(page: Page, appName: string): Promise<string[]> {
  const errors: string[] = [];

  page.on("console", (msg) => {
    if (msg.type() === "error") {
      const text = msg.text();
      errors.push(text);
      console.log(`[${appName}:console:error] ${text}`);
    }
  });

  page.on("pageerror", (err) => {
    errors.push(err.message);
    console.log(`[${appName}:pageerror] ${err.message}`);
  });

  return errors;
}

export function isFatalError(error: string): boolean {
  const fatalPatterns = [
    /Failed to resolve component/,
    /is not a function/,
    /\[Vue warn\].*is not a function/,
    /Cannot read propert/,
    /ReferenceError/,
    /Uncaught TypeError/,
    /Uncaught ReferenceError/,
    /Uncaught SyntaxError/,
    /Failed to fetch dynamically imported module/,
    /ChunkLoadError/,
  ];
  const ignorePatterns = [
    /Failed to load resource/,
    /net::ERR_/,
    /ECONNREFUSED/,
    /is not defined.*\$pinia/,
    // Vite dep optimizer triggers page reload, causing transient import failures
    /Failed to fetch dynamically imported module.*node_modules/,
  ];
  if (ignorePatterns.some((p) => p.test(error))) return false;
  return fatalPatterns.some((p) => p.test(error));
}

export async function collectHydrationErrors(page: Page): Promise<string[]> {
  const errors: string[] = [];
  const hydrationPatterns = [
    /Hydration.*mismatch/i,
    /hydration.*failed/i,
    /hydration.*node/i,
    /\[Vue warn\].*Hydration/i,
  ];

  page.on("console", (msg) => {
    const text = msg.text();
    if (hydrationPatterns.some((p) => p.test(text))) {
      errors.push(text);
      console.log(`[hydration:error] ${text}`);
    }
  });

  return errors;
}

export async function verifyScopedCssAttributes(page: Page): Promise<number> {
  return page.evaluate(() => {
    return (
      document.querySelectorAll("[data-v-]").length ||
      Array.from(document.querySelectorAll("*")).filter((el) =>
        Array.from(el.attributes).some((attr) => attr.name.startsWith("data-v-")),
      ).length
    );
  });
}

export async function getComputedStyleValue(
  page: Page,
  selector: string,
  property: string,
): Promise<string> {
  return page.evaluate(
    ({ sel, prop }) => {
      const el = document.querySelector(sel);
      if (!el) return "";
      return window.getComputedStyle(el).getPropertyValue(prop);
    },
    { sel: selector, prop: property },
  );
}

export async function verifySSRContent(page: Page, url: string): Promise<string> {
  const response = await page.request.get(url);
  return response.text();
}
