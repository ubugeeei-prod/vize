import test from "node:test";
import { proveReconnection } from "./hosted-vrt-reconnect-law.fixtures.ts";

void test(
  "authentic old hosted renderer fails the empty-pane law after real400/401/reconnect",
  { skip: process.env.VIZE_MUSEA_NATIVE_BROWSER_TESTS !== "1", timeout: 180000 },
  async (t) => {
    await proveReconnection(t, "before");
  },
);

void test(
  "public HTTPS native VRT clears stale results and errors across a real session authority change",
  { skip: process.env.VIZE_MUSEA_NATIVE_BROWSER_TESTS !== "1", timeout: 180000 },
  async (t) => {
    await proveReconnection(t, "fixed");
  },
);
