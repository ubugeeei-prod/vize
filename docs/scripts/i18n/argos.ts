import { spawn } from "node:child_process";
import { createInterface } from "node:readline";

export function createArgosClient(python: string) {
  const script = `
import json
import sys
import argostranslate.translate

LOCALE_MAP = {"zh-CN": "zh", "pt-BR": "pt"}
for line in sys.stdin:
    request = json.loads(line)
    try:
        target = LOCALE_MAP.get(request["locale"], request["locale"])
        translated = argostranslate.translate.translate(request["text"], "en", target)
        response = {"id": request["id"], "translated": translated}
    except Exception as error:
        response = {"id": request["id"], "error": str(error)}
    print(json.dumps(response, ensure_ascii=False), flush=True)
`;
  const child = spawn(python, ["-u", "-c", script], { stdio: ["pipe", "pipe", "inherit"] });
  const pending = new Map<
    number,
    { resolve: (text: string) => void; reject: (error: unknown) => void }
  >();
  let nextId = 0;
  createInterface({ input: child.stdout }).on("line", (line) => {
    const response = JSON.parse(line) as { id: number; error?: string; translated: string };
    const request = pending.get(response.id);
    if (!request) return;
    pending.delete(response.id);
    if (response.error) request.reject(new Error(response.error));
    else request.resolve(response.translated);
  });
  child.on("exit", (code) => {
    for (const request of pending.values()) {
      request.reject(new Error(`Argos translation process exited with code ${code}`));
    }
    pending.clear();
  });

  return {
    translate(text: string, locale: string): Promise<string> {
      return new Promise<string>((resolveTranslation, rejectTranslation) => {
        const id = nextId;
        nextId += 1;
        pending.set(id, { resolve: resolveTranslation, reject: rejectTranslation });
        child.stdin.write(`${JSON.stringify({ id, text, locale })}\n`);
      });
    },
    close() {
      child.stdin.end();
    },
  };
}
