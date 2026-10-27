// Embedded by the native loader for the CLI config cache. Records the inputs an
// evaluation reads outside its import graph: environment variables, filesystem
// reads and the node_modules routes of bare imports.
import inputFs from "node:fs";
import * as inputModule from "node:module";
import {
  dirname as inputDirectory,
  isAbsolute as inputIsAbsolute,
  join as inputJoin,
  resolve as inputResolve,
} from "node:path";
import { fileURLToPath as inputFileURLToPath } from "node:url";

const readCalls = [
  "access",
  "accessSync",
  "createReadStream",
  "exists",
  "existsSync",
  "lstat",
  "lstatSync",
  "open",
  "openSync",
  "readFile",
  "readFileSync",
  "readlink",
  "readlinkSync",
  "realpath",
  "realpathSync",
  "stat",
  "statSync",
];
const listCalls = ["opendir", "opendirSync", "readdir", "readdirSync"];
const promiseReadCalls = ["access", "lstat", "open", "readFile", "readlink", "realpath", "stat"];
const promiseListCalls = ["opendir", "readdir"];
const opaqueCalls = ["glob", "globSync"];

/** Start recording; the returned function stops and reports what was read. */
export function recordConfigInputs() {
  // Variable name -> whether its value (not only its presence) was read.
  const env = new Map();
  const files = new Map();
  const routes = new Set();
  let envEnumerated = false;
  let opaque = false;
  let active = true;
  const originalLstat = inputFs.lstatSync;
  const note = (path, listing) => {
    if (!active) return;
    let value = path;
    if (value instanceof URL) value = value.protocol === "file:" ? inputFileURLToPath(value) : "";
    else if (Buffer.isBuffer(value)) value = value.toString();
    if (typeof value !== "string" || value === "") return;
    if (value.startsWith("file:")) value = inputFileURLToPath(value);
    value = inputResolve(value);
    // The loader's private adapter directory is removed before the process exits.
    if (value.includes("vize-config-module-")) return;
    files.set(value, listing || files.get(value) === true);
  };
  const wrap = (owner, name, listing) => {
    const original = owner[name];
    if (typeof original !== "function") return;
    const wrapped = function (path, ...rest) {
      note(path, listing);
      return original.call(this, path, ...rest);
    };
    // Keep attached helpers such as realpathSync.native and promisify.custom.
    for (const key of Reflect.ownKeys(original)) {
      if (key !== "length" && key !== "name" && key !== "prototype") wrapped[key] = original[key];
    }
    owner[name] = wrapped;
  };
  for (const name of readCalls) wrap(inputFs, name, false);
  for (const name of listCalls) wrap(inputFs, name, true);
  for (const name of ["realpath", "realpathSync"]) {
    if (typeof inputFs[name]?.native === "function") wrap(inputFs[name], "native", false);
  }
  for (const name of promiseReadCalls) wrap(inputFs.promises, name, false);
  for (const name of promiseListCalls) wrap(inputFs.promises, name, true);
  for (const owner of [inputFs, inputFs.promises]) {
    for (const name of opaqueCalls) {
      const original = owner[name];
      if (typeof original !== "function") continue;
      owner[name] = function (...args) {
        if (active) opaque = true;
        return original.apply(this, args);
      };
    }
  }
  inputModule.syncBuiltinESMExports();

  const realEnv = process.env;
  const readKey = (key, value) => {
    if (active && typeof key === "string") env.set(key, value || env.get(key) === true);
  };
  process.env = new Proxy(realEnv, {
    get(target, key) {
      readKey(key, true);
      return Reflect.get(target, key);
    },
    has(target, key) {
      readKey(key, false);
      return Reflect.has(target, key);
    },
    // Enumeration (Object.keys, for...in) also lands here once per variable.
    getOwnPropertyDescriptor(target, key) {
      readKey(key, false);
      return Reflect.getOwnPropertyDescriptor(target, key);
    },
    ownKeys(target) {
      if (active) envEnumerated = true;
      return Reflect.ownKeys(target);
    },
  });

  // Node resolves bare imports below Vite internally, without the public fs.
  // Record each searched node_modules route so a relinked package invalidates.
  const hooks = inputModule.registerHooks?.({
    resolve(specifier, context, nextResolve) {
      const result = nextResolve(specifier, context);
      const name = barePackageName(specifier);
      if (active && name !== undefined && context.parentURL?.startsWith("file:")) {
        let directory = inputDirectory(inputFileURLToPath(context.parentURL));
        for (;;) {
          const route = `${directory}\0${name}`;
          if (routes.has(route)) break;
          routes.add(route);
          const candidate = inputJoin(directory, "node_modules", name);
          note(candidate, false);
          if (originalLstat(candidate, { throwIfNoEntry: false }) !== undefined) {
            note(inputJoin(candidate, "package.json"), false);
            break;
          }
          const parent = inputDirectory(directory);
          if (parent === directory) break;
          directory = parent;
        }
      }
      return result;
    },
  });

  return () => {
    active = false;
    hooks?.deregister();
    return { env: [...env], envEnumerated, opaque, files: [...files] };
  };
}

function barePackageName(specifier) {
  if (
    specifier === "" ||
    specifier.startsWith(".") ||
    specifier.startsWith("#") ||
    inputIsAbsolute(specifier) ||
    /^[a-zA-Z][a-zA-Z\d+.-]*:/.test(specifier) ||
    inputModule.isBuiltin(specifier)
  ) {
    return undefined;
  }
  const parts = specifier.split("/");
  return specifier.startsWith("@") ? parts.slice(0, 2).join("/") : parts[0];
}
