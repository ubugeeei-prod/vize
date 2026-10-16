import assert from "node:assert/strict";

let armed = false;

/** Only the separated mounted/patterned observers promise this boundary.
 * Some other loadRuntime callers intentionally use native source transforms.
 * Refuse existing addon images AND future loader calls, rather than infer
 * that an image census alone proves no native addon was used.
 */
export function forbidMountedRuntimeAddons() {
  const report = process.report.getReport();
  assert.ok(typeof report === "object" && report !== null, "native image report is required");
  const images: unknown = Reflect.get(report, "sharedObjects");
  assert.ok(Array.isArray(images), "native image census is required");
  assert.deepEqual(
    images.filter((image) => typeof image === "string" && /\.node(?:$| \(deleted\)$)/u.test(image)),
    [],
    "mounted observer must not have loaded native addons",
  );
  if (armed) return;
  process.dlopen = (_module, filename) => {
    throw new Error(`mounted observer must not load native addon: ${filename}`);
  };
  armed = true;
}
