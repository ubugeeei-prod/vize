import { createRequire } from "node:module";
import { readFileSync, realpathSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { createServer, type Plugin, type ViteDevServer } from "vite-plus";
import { buildNativeApplication } from "./build.ts";

const packageRoot = fileURLToPath(new URL("../", import.meta.url));
const require = createRequire(import.meta.url);

/** Explicit local-development options for the experimental native host. */
export interface NativeDevelopmentOptions {
  /** Canonical application root; test fixtures may use a temporary copy. @default fixture package root */
  readonly root?: string;
  /** Loopback listener port; zero chooses an available port. @default 0 */
  readonly port?: number;
  /** Existing compiler image for preliminary local checks. @default checkout image */
  readonly compilerPath?: string;
}

function nativeSourceAdmission(): Plugin {
  const native = require("@vizejs/native") as typeof import("@vizejs/native");
  function admit(file: string): void {
    const source = readFileSync(file, "utf8");
    const descriptor = native.parseSfc(source, { filename: file, includeAttrs: true });
    if (
      !descriptor.template ||
      !descriptor.scriptSetup ||
      descriptor.script ||
      descriptor.styles.length !== 1 ||
      descriptor.styles.some(({ content }) => content.trim() !== "")
    ) {
      throw new Error(
        "Native development requires canonical SFC sections and empty native styles.",
      );
    }
  }
  return {
    name: "vize-native-host:development-source-admission",
    enforce: "pre",
    transform(_source, id) {
      const file = id.split("?")[0]?.replace(/(\.vue)\.__vize_style_\d+\.[a-z]+$/, "$1");
      if (file?.endsWith(".vue")) admit(file);
      return undefined;
    },
    hotUpdate: {
      order: "pre",
      handler(options) {
        if (options.file.endsWith(".vue")) admit(options.file);
      },
    },
  };
}

const developmentHtml = `<!doctype html>
<html lang="en"><head><meta charset="utf-8"><title>Vize Native Development</title></head>
<body><script type="module" src="/src/development-entry.ts"></script></body></html>`;

function nativeDevelopmentPage(): Plugin {
  return {
    name: "vize-native-host:development-page",
    configureServer(server) {
      server.middlewares.use((request, response, next) => {
        if (request.url !== "/") return next();
        const address = server.httpServer?.address();
        if (
          !address ||
          typeof address === "string" ||
          request.headers.host !== `127.0.0.1:${address.port}`
        ) {
          response.statusCode = 400;
          response.end("Native development requires its explicit loopback origin.");
          return;
        }
        void server
          .transformIndexHtml("/", developmentHtml)
          .then((html) => {
            response.statusCode = 200;
            response.setHeader("Content-Type", "text/html; charset=utf-8");
            response.setHeader("Cache-Control", "no-store");
            response.setHeader(
              "Content-Security-Policy",
              [
                "default-src 'none'",
                "script-src 'self'",
                `connect-src 'self' ws://127.0.0.1:${address.port}`,
                "style-src 'self' 'unsafe-inline'",
                "base-uri 'none'",
                "frame-ancestors 'none'",
              ].join("; "),
            );
            response.end(html);
          })
          .catch((error: unknown) => next(error));
      });
    },
  };
}

/**
 * Opens a loopback-only development server using the established SFC HMR path.
 * Template-only edits preserve component setup state. Script edits retain the
 * compiler plugin's explicit remount behavior. This does not qualify document
 * reload, reconnection, or physical-device networking.
 */
export async function startNativeDevelopment(options: NativeDevelopmentOptions = {}) {
  const port = options.port ?? 0;
  if (!Number.isInteger(port) || port < 0 || port > 65535) {
    throw new TypeError("Native development port must be an integer from zero through 65535.");
  }
  const application = await buildNativeApplication(
    options.compilerPath ? { compilerPath: options.compilerPath } : {},
  );
  const { vize } = await import("@vizejs/vite-plugin");
  const root = realpathSync(path.resolve(options.root ?? packageRoot));
  const server: ViteDevServer = await createServer({
    configFile: false,
    root,
    mode: "development",
    logLevel: "warn",
    plugins: [
      nativeSourceAdmission(),
      nativeDevelopmentPage(),
      vize({
        configMode: false,
        isProduction: false,
        vapor: false,
        customRenderer: true,
        customElements: ["native-view", "native-text", "native-button"],
      }),
    ],
    resolve: { dedupe: ["vue"] },
    server: {
      host: "127.0.0.1",
      port,
      strictPort: true,
      cors: false,
      hmr: { overlay: false },
      fs: { allow: [root, path.resolve(packageRoot, "../..")] },
    },
  });
  try {
    await server.listen();
    const address = server.httpServer?.address();
    if (!address || typeof address === "string")
      throw new Error("Native development needs a TCP listener.");
    const origin = `http://127.0.0.1:${address.port}/`;
    const source = readFileSync(path.join(root, "src/App.vue"), "utf8");
    return { server, origin, source, application: application.receipt };
  } catch (error) {
    await server.close();
    throw error;
  }
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const { parseArgs } = await import("node:util");
  const { values } = parseArgs({ options: { port: { type: "string", default: "0" } } });
  const { server, origin, application } = await startNativeDevelopment({
    port: Number(values.port),
    ...(process.env.VIZE_NATIVE_HOST_COMPILER_PATH
      ? { compilerPath: process.env.VIZE_NATIVE_HOST_COMPILER_PATH }
      : {}),
  });
  console.log(JSON.stringify({ event: "native-development-listening", origin, application }));
  const close = () => {
    void server.close().then(() => process.exit(0));
  };
  process.once("SIGINT", close);
  process.once("SIGTERM", close);
}
