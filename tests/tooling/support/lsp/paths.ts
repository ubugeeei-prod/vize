import path from "node:path";
import { fileURLToPath } from "node:url";

/**
 * Repository root used by LSP smoke tests and their support modules.
 *
 * Keeping this path in one module avoids each helper guessing how many
 * directory segments it needs to climb, which matters because these smoke tests
 * launch the real `vize lsp` binary and create temporary workspaces under the
 * repository-local `__agent_only` directory.
 */
export const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../../..");
