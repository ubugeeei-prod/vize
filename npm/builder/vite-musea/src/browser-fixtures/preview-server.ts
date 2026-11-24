import { readFile } from "node:fs/promises";
import { createServer } from "node:http";

export async function createPreviewFixtureServer(html: string, scriptName: string) {
  const script = await readFile(new URL(scriptName, import.meta.url));
  const scriptPath = `/${scriptName}`;
  const page = `${html}<script src="${scriptPath}"></script>`;

  return createServer((request, response) => {
    if (request.url === scriptPath) {
      response.writeHead(200, { "Content-Type": "text/javascript" }).end(script);
      return;
    }
    response
      .writeHead(200, {
        "Content-Type": "text/html",
        "Content-Security-Policy": "script-src 'self'",
      })
      .end(page);
  });
}
