// Load Vue before the traps so only the entry under test is observed.
import "vue";

for (const name of ["window", "document", "navigator"]) {
  Object.defineProperty(globalThis, name, {
    configurable: true,
    get() {
      throw new Error(`[import-side-effect] module-level access to globalThis.${name}`);
    },
  });
}
await import(process.argv[2]);
