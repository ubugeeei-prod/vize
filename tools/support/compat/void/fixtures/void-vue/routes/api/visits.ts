import { defineHandler } from "void";
import { kv } from "void/kv";

export const GET = defineHandler(async () => {
  const visits = ((await kv.get<number>("visits")) ?? 0) + 1;
  await kv.put("visits", visits);
  return { visits };
});
