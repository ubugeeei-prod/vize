(() => {
  // Chromium may re-run the context initializer in a popup's existing realm.
  // Retain the original inventory and wrappers when that realm is initialized.
  if (Object.hasOwn(window, "__toolbarMessageListenerNames")) return;
  const listeners = new Set();
  for (const method of ["addEventListener", "removeEventListener"]) {
    const original = window[method];
    Object.defineProperty(window, method, {
      configurable: true,
      enumerable: true,
      writable: true,
      value(...args) {
        const result = Reflect.apply(original, this, args);
        if (this === window && args[0] === "message" && args[1] != null) {
          if (method === "addEventListener") listeners.add(args[1]);
          else listeners.delete(args[1]);
        }
        return result;
      },
    });
  }
  Object.defineProperty(window, "__toolbarMessageListenerCount", {
    configurable: true,
    get() {
      return listeners.size;
    },
  });
  Object.defineProperty(window, "__toolbarMessageListenerNames", {
    configurable: true,
    get() {
      return [...listeners]
        .map((listener) =>
          typeof listener === "function"
            ? listener.name || "(anonymous)"
            : listener?.handleEvent?.name || "(object listener)",
        )
        .sort((left, right) => (left === right ? 0 : left < right ? -1 : 1));
    },
  });
})();
