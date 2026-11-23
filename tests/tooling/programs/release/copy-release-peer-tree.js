import { copyTree } from "../../release/release-smoke-parity-peer.ts";

try {
  copyTree(process.argv[2], process.argv[3]);
} catch (error) {
  process.stderr.write(error.message);
  process.exitCode = 1;
}
