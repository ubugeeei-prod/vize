import { nextTick } from "vue";

export async function settle(): Promise<void> {
  await Promise.resolve();
  await nextTick();
}

export function createShareFile(name = "share.txt"): File {
  return new File(["shared content"], name, { type: "text/plain" });
}
