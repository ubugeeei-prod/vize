export type Loader = (this: unknown, ...args: unknown[]) => unknown;
export function observeLoader(
  original: Loader,
  matches: (args: unknown[]) => boolean,
  emit: (event: "attempt" | "returned" | "failed") => void,
  verify: () => void,
): Loader {
  return function (this: unknown, ...args: unknown[]): unknown {
    if (!matches(args)) return Reflect.apply(original, this, args);
    verify();
    emit("attempt");
    try {
      const result = Reflect.apply(original, this, args);
      verify();
      emit("returned");
      return result;
    } catch (error) {
      emit("failed");
      throw error;
    }
  };
}
