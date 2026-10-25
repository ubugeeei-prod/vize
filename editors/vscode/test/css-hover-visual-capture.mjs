import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { assertProviderPicker, assertRenderedCss } from "./css-hover-visual-contract.mjs";

const pause = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
export async function until(produce, label, timeoutMs = 60_000) {
  const deadline = Date.now() + timeoutMs;
  let lastError;
  while (Date.now() < deadline) {
    try {
      const value = await produce();
      if (value) return value;
    } catch (error) {
      lastError = error;
    }
    await pause(100);
  }
  throw new Error(`Timed out: ${label}`, { cause: lastError });
}

// Read-only DOM capture; it does not create or restyle Markdown or editor nodes.
async function snapshot(page, session, destination, id) {
  const observed = await page.evaluate(() => {
    const visible = (node) =>
      node.getBoundingClientRect().width > 0 && node.getBoundingClientRect().height > 0;
    const hovers = [...document.querySelectorAll(".monaco-hover")].filter(visible);
    const hover = hovers.find((node) => node.querySelector(".rendered-markdown")) ?? hovers[0];
    const tokenNodes =
      hover?.querySelectorAll(
        ".rendered-markdown code span, .rendered-markdown .monaco-tokenized-source span",
      ) ?? [];
    const decorations = [...document.querySelectorAll(".colorpicker-color-decoration")].filter(
      visible,
    );
    const picker = [...document.querySelectorAll(".colorpicker-widget")].find(visible);
    return {
      title: document.title,
      url: location.href,
      hoverText: hover?.innerText ?? "",
      hoverHtml: hover?.outerHTML ?? null,
      tokens: [...tokenNodes].map((node) => ({
        text: node.textContent,
        className: node.className,
        color: getComputedStyle(node).color,
      })),
      codeBlocks: [...(hover?.querySelectorAll(".monaco-tokenized-source") ?? [])].map((block) => ({
        text: block.textContent,
        html: block.outerHTML,
        tokens: [...block.querySelectorAll("span")].map((node) => ({
          text: node.textContent,
          className: node.className,
          color: getComputedStyle(node).color,
        })),
      })),
      links: [...(hover?.querySelectorAll("a") ?? [])].map((node) => ({
        text: node.textContent,
        href: node.href,
        attributeHref: node.getAttribute("href"),
        dataHref: node.getAttribute("data-href"),
      })),
      decorations: decorations.map((node, index) => ({
        index,
        className: node.className,
        backgroundColor: getComputedStyle(node).backgroundColor,
        html: node.outerHTML,
        line: node.closest(".view-line")?.textContent,
        rect: node.getBoundingClientRect().toJSON(),
      })),
      pickerOriginalColor: picker
        ? getComputedStyle(picker.querySelector(".original-color")).backgroundColor
        : null,
      pickerPickedColor: picker
        ? getComputedStyle(picker.querySelector(".picked-color")).backgroundColor
        : null,
      pickerPresentation: picker?.querySelector(".picked-color-presentation")?.textContent ?? "",
      standalonePicker: Boolean(picker?.querySelector(".standalone-colorpicker")),
      pickerHtml: picker?.outerHTML ?? null,
    };
  });
  fs.writeFileSync(
    path.join(destination, `${id}.dom.json`),
    `${JSON.stringify(observed, null, 2)}\n`,
  );
  fs.writeFileSync(path.join(destination, `${id}.html`), await page.content());
  fs.writeFileSync(
    path.join(destination, `${id}.aria.txt`),
    await page.locator("body").ariaSnapshot(),
  );
  const dom = await session.send("DOMSnapshot.captureSnapshot", {
    computedStyles: ["color", "background-color", "display", "visibility"],
  });
  fs.writeFileSync(path.join(destination, `${id}.cdp-dom.json`), JSON.stringify(dom));
  await page.screenshot({ path: path.join(destination, `${id}.png`) });
  return observed;
}

export async function captureCssHover({ chromium, port, destination }) {
  const endpoint = `http://127.0.0.1:${port}`;
  await until(async () => (await fetch(`${endpoint}/json/version`)).ok, "Electron CDP endpoint");
  const browser = await chromium.connectOverCDP(endpoint);
  const network = [];
  const assessments = [];
  try {
    const page = await until(
      () =>
        browser
          .contexts()
          .flatMap((context) => context.pages())
          .find((candidate) => candidate.url().includes("workbench")),
      "VS Code workbench page",
    );
    const session = await page.context().newCDPSession(page);
    session.on("Network.requestWillBeSent", (event) => network.push(event));
    await session.send("Network.enable");
    // Hover documentation and syntax tokenization must work without fetching docs.
    await session.send("Network.emulateNetworkConditions", {
      offline: true,
      latency: 0,
      downloadThroughput: 0,
      uploadThroughput: 0,
    });
    fs.writeFileSync(path.join(destination, "renderer-ready"), "offline\n");
    for (const id of ["property", "named", "picker"]) {
      await until(
        () => {
          if (fs.existsSync(path.join(destination, "host-error.json")))
            throw new Error(fs.readFileSync(path.join(destination, "host-error.json"), "utf8"));
          return (
            JSON.parse(fs.readFileSync(path.join(destination, "phase.json"), "utf8")).id === id
          );
        },
        `host phase ${id}`,
        180_000,
      );
      // Preserve current real DOM before locating/interacting with a decorator.
      await snapshot(page, session, destination, `${id}-before`);
      try {
        if (id === "picker") {
          const observed = await until(
            async () => {
              const nodes = page.locator(".colorpicker-color-decoration");
              const backgrounds = await nodes.evaluateAll((items) =>
                items.map((node) => getComputedStyle(node).backgroundColor),
              );
              const index = backgrounds.indexOf("rgb(51, 102, 153)");
              return index < 0 ? null : { nodes, index };
            },
            "Vize provider-backed color decorator",
            15_000,
          );
          // Refresh the snapshot immediately before using this live locator.
          await snapshot(page, session, destination, "picker-decorator");
          await observed.nodes.nth(observed.index).hover();
          await until(
            async () => {
              const result = await snapshot(page, session, destination, "picker");
              assertProviderPicker(result);
              return result;
            },
            "real provider color picker",
            15_000,
          );
        } else {
          const result = await until(
            async () => {
              const result = await snapshot(page, session, destination, id);
              return result.hoverText.includes(
                id === "property" ? "display" : "CSS value for color",
              ) && result.tokens.length > 0
                ? result
                : null;
            },
            `${id} hover CSS tokenization`,
            15_000,
          );
          // Keep the real code fence visible even when the provider-backed
          // picker fills the top of a named-color popup.
          await snapshot(page, session, destination, `${id}-before-fence-scroll`);
          const fences = page.locator(".monaco-hover .monaco-tokenized-source");
          const executableExample = fences.filter({ hasText: ".example {" });
          await (
            (await executableExample.count()) ? executableExample.first() : fences.first()
          ).scrollIntoViewIfNeeded();
          await snapshot(page, session, destination, `${id}-fence`);
          assertRenderedCss(result);
        }
        assessments.push({ id, status: "pass" });
      } catch (error) {
        await snapshot(page, session, destination, `${id}-failed`);
        assessments.push({
          id,
          status: "fail",
          message: error.message,
          cause: String(error.cause ?? ""),
          stack: error.stack,
        });
      }
      fs.writeFileSync(
        path.join(destination, "visual-assessments.json"),
        `${JSON.stringify(assessments, null, 2)}\n`,
      );
      await page.keyboard.press("Escape");
      fs.writeFileSync(path.join(destination, `${id}.ack`), "captured\n");
    }
    await until(() => fs.existsSync(path.join(destination, "complete.json")), "host completion");
    assert.ok(
      assessments.every((result) => result.status === "pass"),
      `Actual packaged visual gaps: ${JSON.stringify(assessments)}`,
    );
  } finally {
    fs.writeFileSync(
      path.join(destination, "network.json"),
      `${JSON.stringify({ offlineDuringCapture: true, requests: network }, null, 2)}\n`,
    );
    await browser.close();
  }
}
