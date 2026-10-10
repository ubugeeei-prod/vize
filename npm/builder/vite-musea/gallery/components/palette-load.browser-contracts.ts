import type { PaletteApiResponse } from "../api";
import { usePalette } from "../composables/usePalette";

export interface Fixture {
  old: PaletteApiResponse;
  current: PaletteApiResponse;
}

export async function runPaletteLoadOrder(input: Fixture) {
  const originalFetch = window.fetch;
  const requests: Array<{
    url: string;
    resolve: (response: Response) => void;
    reject: (error: Error) => void;
  }> = [];
  window.fetch = (resource, options) => {
    const url =
      typeof resource === "string"
        ? resource
        : resource instanceof URL
          ? resource.href
          : resource.url;
    if (!url.includes("__palette_load_race__")) return originalFetch(resource, options);
    return new Promise<Response>((resolve, reject) => requests.push({ url, resolve, reject }));
  };
  const snapshot = (state: ReturnType<typeof usePalette>) =>
    JSON.parse(
      JSON.stringify({
        palette: state.palette.value,
        values: state.values.value,
        customProps: state.customProps.value,
        deletedPaletteProps: [...state.deletedPaletteProps.value],
        loading: state.loading.value,
        error: state.error.value,
      }),
    );
  const complete = (index: number, palette: PaletteApiResponse) =>
    requests[index].resolve(new Response(JSON.stringify(palette)));
  const oldPath = "__palette_load_race__/previous.vue";
  const currentPath = "__palette_load_race__/selected.vue";
  try {
    const edited = usePalette();
    const oldSuccess = edited.load(oldPath);
    const currentSuccess = edited.load(currentPath);
    complete(1, input.current);
    await currentSuccess;
    edited.setValue("label", "Selected edit");
    edited.addProp("extra", "number", 7);
    edited.removeProp("tone");
    const beforeOldSuccess = snapshot(edited);
    complete(0, input.old);
    await oldSuccess;
    const afterOldSuccess = snapshot(edited);

    const successful = usePalette();
    const oldError = successful.load(oldPath);
    const currentResult = successful.load(currentPath);
    complete(3, input.current);
    await currentResult;
    const beforeOldError = snapshot(successful);
    requests[2].reject(new Error("Previous component request failed"));
    await oldError;
    const afterOldError = snapshot(successful);

    const pending = usePalette();
    const firstPending = pending.load(oldPath);
    const lastPending = pending.load(currentPath);
    complete(4, input.old);
    await firstPending;
    const whileCurrentPending = snapshot(pending);
    complete(5, input.current);
    await lastPending;
    const afterCurrentPending = snapshot(pending);

    const failed = usePalette();
    const obsolete = failed.load(oldPath);
    const selectedFailure = failed.load(currentPath);
    requests[7].resolve(new Response("Unavailable", { status: 503, statusText: "Unavailable" }));
    await selectedFailure;
    const beforeObsolete = snapshot(failed);
    complete(6, input.old);
    await obsolete;
    const afterObsolete = snapshot(failed);

    const repeated = usePalette();
    const a1 = repeated.load(currentPath);
    const b = repeated.load(oldPath);
    const a2 = repeated.load(currentPath);
    complete(10, input.current);
    await a2;
    repeated.setValue("label", "Newest visit edit");
    const beforeRepeated = snapshot(repeated);
    complete(8, input.old);
    complete(9, input.old);
    await Promise.all([a1, b]);
    return {
      beforeOldSuccess,
      afterOldSuccess,
      beforeOldError,
      afterOldError,
      whileCurrentPending,
      afterCurrentPending,
      beforeObsolete,
      afterObsolete,
      beforeRepeated,
      afterRepeated: snapshot(repeated),
      requests: requests.map(({ url }) => url),
    };
  } finally {
    window.fetch = originalFetch;
  }
}
