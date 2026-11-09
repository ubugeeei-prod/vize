import { buildEsbuild } from "./esbuild.ts";
import { buildWebpack } from "./webpack.ts";

await Promise.all([buildEsbuild(), buildWebpack()]);
