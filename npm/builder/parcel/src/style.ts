import { Transformer } from "@parcel/plugin";
import { scopeViteCssForPipeline, transformViteCssVarsForPipeline } from "@vizejs/native";
import { editMap } from "./maps.ts";

/** Run before Parcel's CSS pipeline, after its Sass/Less/Stylus pipeline. */
export default new Transformer({
  async transform({ asset, options }) {
    if (asset.meta.vizeStyleApplied) return [asset];
    const metadata = asset.meta.vizeStyle as
      | { scope?: string; variables?: string; productionFilename?: string }
      | undefined;
    const scope = metadata?.scope ?? asset.query.get("vize-scope");
    const variables = metadata?.variables ?? asset.query.get("vize-vars");
    if (!scope && !variables) return [asset];
    for (const value of [scope, variables]) {
      if (value && !/^data-v-[a-f0-9]+$/.test(value)) {
        throw new Error("Invalid Vize style scope identifier.");
      }
    }
    const before = await asset.getCode();
    const productionFilename =
      metadata?.productionFilename ?? asset.query.get("vize-owner") ?? undefined;
    let after = variables
      ? transformViteCssVarsForPipeline(before, variables, productionFilename)
      : before;
    if (scope) after = scopeViteCssForPipeline(after, scope);
    const map = editMap(
      before,
      after,
      (await asset.getMap()) ?? undefined,
      asset.filePath,
      options.projectRoot,
    );
    asset.setCode(after);
    asset.setMap(map ?? null);
    // Parcel can revisit a pipeline. Never apply the scope transformation twice.
    asset.meta.vizeStyleApplied = true;
    return [asset];
  },
});
