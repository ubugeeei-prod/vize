export async function observeReversedCssOrder() {
              const globals = [...document.head.querySelectorAll("style[data-vite-dev-id]")].filter(
                (node) =>
                  (node.getAttribute("data-vite-dev-id") ?? "").includes("/assets/styles/main.css"),
              );
              if (!globals.length) throw Error("original global stylesheet missing");
              for (const node of globals) document.head.append(node);
              // The complete injected global sheet moves; original HTTP links
              // and every rule stay intact. Let style invalidation settle before
              // reading both computed properties in the same observation.
              await new Promise(requestAnimationFrame);
              await new Promise(requestAnimationFrame);
              return {
                html: document.documentElement.outerHTML,
                headings: [...document.querySelectorAll("h2")].map((heading) => ({
                  html: heading.outerHTML,
                  fontSize: getComputedStyle(heading).fontSize,
                  lineHeight: getComputedStyle(heading).lineHeight,
                })),
                sheets: [...document.styleSheets].map((sheet) => ({
                  href: sheet.href,
                  owner: sheet.ownerNode.outerHTML,
                  rules: [...sheet.cssRules].map((rule) => rule.cssText),
                })),
              };
            }
