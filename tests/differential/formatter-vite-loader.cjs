const fs = require("node:fs");
const crypto = require("node:crypto");

const destination = process.env.VIZE_VITE_CLI_LOAD_CAPTURE;
if (!destination) throw new Error("instrumented CLI requires a load capture destination");
const hash = (file) => crypto.createHash("sha256").update(fs.readFileSync(file)).digest("hex");
const actual = Object.getOwnPropertyDescriptor(process, "dlopen").value;
const loads = [];
process.dlopen = function (...args) {
  const [, filename] = args;
  const load = {
    filename,
    realpath: null,
    beforeSha256: null,
    afterSha256: null,
    completed: false,
    exportDescriptors: null,
    error: null,
    observationErrors: [],
  };
  loads.push(load);
  try {
    load.realpath = fs.realpathSync(filename);
    load.beforeSha256 = hash(filename);
  } catch (error) {
    load.observationErrors.push(error.message);
  }
  try {
    const result = Reflect.apply(actual, this, args);
    load.completed = true;
    try {
      load.exportDescriptors = Reflect.ownKeys(args[0].exports).map((key) => {
        const descriptor = Object.getOwnPropertyDescriptor(args[0].exports, key);
        return {
          key: typeof key === "symbol" ? { symbol: key.description } : key,
          enumerable: descriptor.enumerable,
          configurable: descriptor.configurable,
          ...(Object.hasOwn(descriptor, "value")
            ? { writable: descriptor.writable, valueType: typeof descriptor.value }
            : { getType: typeof descriptor.get, setType: typeof descriptor.set }),
        };
      });
    } catch (error) {
      load.observationErrors.push(error.message);
    }
    return result;
  } catch (error) {
    load.error = { name: error.name, message: error.message };
    throw error;
  } finally {
    // Observation failures must not replace the actual return or thrown Error.
    // Missing/incomplete custody is rejected by the outer report admission.
    try {
      load.afterSha256 = hash(filename);
    } catch (error) {
      load.observationErrors.push(error.message);
    }
    try {
      fs.writeFileSync(destination, JSON.stringify({ loads }));
    } catch {}
  }
};
