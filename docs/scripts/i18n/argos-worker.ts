import { parentPort } from "node:worker_threads";
import { python } from "pythonia";

const port = parentPort;
if (!port) throw new Error("The Argos worker requires a parent");
const locales: Record<string, string> = { "zh-CN": "zh", "pt-BR": "pt" };
const translator = (await python("argostranslate.translate")) as {
  translate(text: string, from: string, to: string): Promise<string>;
};

type Request = { id: number; locale: string; text: string };
let work = Promise.resolve();
port.on("message", (request: Request | null) => {
  work = work.then(async () => {
    if (request === null) {
      python.exit();
      port.close();
      return;
    }
    const { id, locale, text } = request;
    try {
      const translated = await translator.translate(text, "en", locales[locale] ?? locale);
      port.postMessage({ id, translated });
    } catch (error) {
      const message =
        error instanceof Error ? error.message || error.stack || error.name : String(error);
      port.postMessage({ id, error: message });
    }
  });
});
