import { serializeScriptValue } from "../security.js";

export function generateGalleryModule(basePath: string): string {
  return `
export const basePath = ${serializeScriptValue(basePath)};
export async function loadArts() {
  const res = await fetch(basePath + '/api/arts');
  return res.json();
}
`;
}
