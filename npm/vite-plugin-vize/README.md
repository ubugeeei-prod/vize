# @vizejs/vite-plugin

High-performance native Vite plugin for Vue SFC compilation powered by [Vize](https://github.com/ubugeeei/vize).

## Features

- **Native Performance**: Uses Rust-based compiler via Node.js native bindings (NAPI)
- **Pre-compilation**: All `.vue` files are compiled at server startup for instant module resolution
- **Virtual Modules**: Compiled code is served from memory as virtual modules
- **HMR Support**: Hot Module Replacement with automatic re-compilation on file changes
- **Vapor Mode**: Optional support for Vue Vapor mode compilation

## Installation

```bash
# npm
npm install @vizejs/vite-plugin vize

# pnpm
pnpm add @vizejs/vite-plugin vize

# yarn
yarn add @vizejs/vite-plugin vize
```

## Usage

### Vite

```ts
// vite.config.ts
import { defineConfig } from "vite";
import vize from "@vizejs/vite-plugin";

export default defineConfig({
  plugins: [
    vize({
      // options
    }),
  ],
});
```

### Shared config

`vize.config.ts`

```ts
import { defineConfig } from "vize";

export default defineConfig({
  compiler: {
    sourceMap: true,
  },
  vite: {
    scanPatterns: ["src/**/*.vue"],
  },
});
```

`vize.config.pkl`

```pkl
amends "node_modules/vize/pkl/vize.pkl"

compiler {
  sourceMap = true
}

vite {
  scanPatterns = new Listing {
    "src/**/*.vue"
  }
}
```

`@vizejs/vite-plugin` loads the same `vize.config.ts`, `vize.config.js`, `vize.config.mjs`,
`vize.config.pkl`, and `vize.config.json` files as the `vize` npm CLI. Importing
`defineConfig` from `@vizejs/vite-plugin` still works, but `vize` is the shared entry point.

### Nuxt

For Nuxt 3, add the plugin to your `nuxt.config.ts`:

```ts
// nuxt.config.ts
import vize from "@vizejs/vite-plugin";

export default defineNuxtConfig({
  vite: {
    plugins: [
      vize({
        // Exclude Nuxt's internal .vue files if needed
        exclude: [/node_modules/, /#/, /\.nuxt/],
      }),
    ],
  },

  // Disable the default Vue plugin
  vue: {
    propsDestructure: false,
  },
});
```

**Note**: When using with Nuxt, you may need to disable Nuxt's built-in Vue plugin to avoid conflicts:

```ts
// nuxt.config.ts
export default defineNuxtConfig({
  hooks: {
    "vite:extendConfig": (config) => {
      // Remove @vitejs/plugin-vue from plugins
      config.plugins = config.plugins?.filter(
        (p) => p && (Array.isArray(p) ? p[0] : p).name !== "vite:vue",
      );
    },
  },
  vite: {
    plugins: [vize()],
  },
});
```

## Options

```ts
interface VizeNativeOptions {
  /**
   * Files to include in compilation
   * @default /\.vue$/
   */
  include?: string | RegExp | (string | RegExp)[];

  /**
   * Files to exclude from compilation
   * @default /node_modules/
   */
  exclude?: string | RegExp | (string | RegExp)[];

  /**
   * Force production mode
   * @default auto-detected from Vite config
   */
  isProduction?: boolean;

  /**
   * Enable SSR mode
   * @default false
   */
  ssr?: boolean;

  /**
   * Enable source map generation
   * @default true in development, false in production
   */
  sourceMap?: boolean;

  /**
   * Enable Vapor mode compilation
   * @default false
   */
  vapor?: boolean;

  /**
   * Root directory to scan for .vue files
   * @default Vite's root
   */
  root?: string;

  /**
   * Glob patterns to scan for .vue files during pre-compilation
   * @default ['**\/*.vue']
   */
  scanPatterns?: string[];

  /**
   * Glob patterns to ignore during pre-compilation
   * @default ['node_modules/**', 'dist/**', '.git/**']
   */
  ignorePatterns?: string[];
}
```

## How It Works

### Pre-compilation at Startup

When the Vite dev server starts (or build begins), the plugin:

1. Scans the project root for all `.vue` files matching the configured patterns
2. Compiles each file using the native Vize compiler
3. Stores the compiled JavaScript and CSS in an in-memory cache

This approach leverages Vize's exceptional performance - compiling 15,000 SFC files in under 500ms with multi-threading.

### Virtual Module Resolution

When Vite requests a `.vue` file:

1. The plugin intercepts the module resolution
2. Returns the pre-compiled code from cache (or compiles on-demand if not cached)
3. CSS is injected inline with deduplication support

### HMR (Hot Module Replacement)

When a `.vue` file changes:

1. The plugin detects the change via `handleHotUpdate`
2. Re-compiles only the changed file
3. Updates the cache
4. Vite handles the rest of the HMR flow

## Performance

Vize's native compiler is significantly faster than the official Vue compiler:

| Benchmark (15,000 SFCs) | @vue/compiler-sfc | Vize  | Speedup  |
| ----------------------- | ----------------- | ----- | -------- |
| Single-threaded         | 10.43s            | 6.06s | **1.7x** |
| Multi-threaded          | 3.45s             | 612ms | **5.6x** |

## Comparison with vite-plugin-vize

| Feature         | vite-plugin-vize | vite-plugin-vize      |
| --------------- | ---------------- | --------------------- |
| Compiler        | WASM             | Native (NAPI)         |
| Pre-compilation | No               | Yes                   |
| Module Loading  | Transform        | Virtual Module (Load) |
| Performance     | Fast             | Fastest               |
| Platform        | Any              | Node.js only          |

Use `vite-plugin-vize` (WASM-based) when you need:

- Browser compatibility (e.g., StackBlitz, WebContainers)
- Platform-independent deployment

Use `vite-plugin-vize` when you need:

- Maximum performance
- Server-side only (standard Node.js environment)

## Requirements

- Node.js 18+
- Vite 5.0+ / 6.0+ / 7.0+
- Vue 3.x

## License

MIT
