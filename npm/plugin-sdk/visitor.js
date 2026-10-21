// Capability-only visitor authoring contracts. Native pipeline registration is
// deliberately separate so importing this module cannot affect compiler paths.
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { SDK_FINGERPRINT } from "./identity.js";

export const VISITOR_API_VERSION = 1;
export const VISITOR_CAPABILITIES = Object.freeze([
  "sfc",
  "template",
  "script",
  "style",
  "virtual-ts",
  "diagnostics",
  "lint",
  "format",
  "devtools-trace",
]);

const CAPABILITY_ORDER = new Map(
  VISITOR_CAPABILITIES.map((capability, index) => [capability, index]),
);
const VISITOR_RUNTIME_FINGERPRINT = createHash("sha256")
  .update(readFileSync(new URL("visitor.js", import.meta.url)))
  .digest("hex");
const plugins = new WeakSet();

export class VisitorPluginValidationError extends TypeError {
  constructor(code, field, message) {
    super(`[visitor/${code}] ${field}: ${message}`);
    this.name = "VisitorPluginValidationError";
    this.code = code;
    this.field = field;
  }
}

/**
 * Define an immutable v1 visitor descriptor without registering it in a Vize
 * runtime. A host must explicitly negotiate the descriptor before invocation.
 */
export function defineVisitorPlugin(definition) {
  record(definition, "definition", "invalid-definition");
  const name = nonempty(definition.name, "name", "invalid-name");
  const version = nonempty(definition.version, "version", "invalid-version");
  apiVersion(definition.apiVersion, "apiVersion");
  const capabilities = capabilitiesOf(definition.capabilities, "capabilities", true);
  const visitors = visitorsOf(definition.visitors, capabilities);
  const cacheInputs = cacheInputsOf(definition.cacheInputs);
  const callbacks = capabilities.map((capability) => [capability, String(visitors[capability])]);
  const fingerprint = createHash("sha256")
    .update(
      JSON.stringify([
        SDK_FINGERPRINT,
        VISITOR_RUNTIME_FINGERPRINT,
        VISITOR_API_VERSION,
        name,
        version,
        capabilities,
        cacheInputs,
        callbacks,
      ]),
    )
    .digest("hex");
  const plugin = immutable({
    kind: "vize.visitor-plugin",
    apiVersion: VISITOR_API_VERSION,
    name,
    version,
    capabilities,
    cacheInputs,
    visitors,
    fingerprint,
  });
  plugins.add(plugin);
  return plugin;
}

/** Validate a host contract and fail closed when any requested capability is absent. */
export function negotiateVisitorPlugin(plugin, host) {
  if (!plugins.has(plugin)) {
    fail("invalid-plugin", "plugin", "must be created by defineVisitorPlugin");
  }
  record(host, "host", "invalid-host-contract");
  apiVersion(host.apiVersion, "host.apiVersion");
  const available = capabilitiesOf(host.capabilities, "host.capabilities", false);
  const supported = new Set(available);
  const missing = plugin.capabilities.filter((capability) => !supported.has(capability));
  if (missing.length > 0) {
    fail(
      "unsupported-capability",
      "host.capabilities",
      `missing ${missing.map((capability) => JSON.stringify(capability)).join(", ")}`,
    );
  }
  return immutable({ apiVersion: VISITOR_API_VERSION, capabilities: [...plugin.capabilities] });
}

/**
 * Invoke one negotiated visitor in an author fixture. This is not a compiler or
 * linter registration path; production hosts own cancellation and isolation.
 */
export function runVisitorFixture(plugin, capability, event) {
  if (!plugins.has(plugin)) {
    fail("invalid-plugin", "plugin", "must be created by defineVisitorPlugin");
  }
  if (!CAPABILITY_ORDER.has(capability)) {
    fail("unknown-capability", "capability", `${JSON.stringify(capability)} is not supported`);
  }
  if (!plugin.capabilities.includes(capability)) {
    fail(
      "undeclared-capability",
      "capability",
      `${JSON.stringify(capability)} is not declared by ${JSON.stringify(plugin.name)}`,
    );
  }
  record(event, "event", "invalid-event");
  apiVersion(event.apiVersion, "event.apiVersion");
  if (event.capability !== capability) {
    fail(
      "invalid-event",
      "event.capability",
      `expected ${JSON.stringify(capability)}, received ${JSON.stringify(event.capability)}`,
    );
  }
  nonempty(event.file, "event.file", "invalid-event");
  if (!("payload" in event)) fail("invalid-event", "event.payload", "is required");
  const result = plugin.visitors[capability](immutable(event));
  if (result?.then) {
    fail(
      "async-visitor",
      `visitors.${capability}`,
      "returned a Promise; asynchronous visitors are unsupported",
    );
  }
  if (result !== undefined) {
    fail("invalid-visitor-result", `visitors.${capability}`, "must return undefined");
  }
}

/** Adapt one negotiated virtual-TypeScript-only descriptor to the native host boundary. */
export function createVirtualTsVisitorHost(plugin) {
  negotiateVisitorPlugin(plugin, {
    apiVersion: VISITOR_API_VERSION,
    capabilities: ["virtual-ts"],
  });
  return immutable({
    name: plugin.name,
    version: plugin.version,
    fingerprint: plugin.fingerprint,
    run(eventJson) {
      if (typeof eventJson !== "string") {
        fail("invalid-event", "event", "must be serialized JSON");
      }
      let event;
      try {
        event = JSON.parse(eventJson);
      } catch {
        fail("invalid-event", "event", "must be valid JSON");
      }
      runVisitorFixture(plugin, "virtual-ts", event);
      return "";
    },
  });
}

/** Adapt one negotiated devtools-trace-only descriptor to the recorder boundary. */
export function createDevtoolsTraceVisitorHost(plugin) {
  negotiateVisitorPlugin(plugin, {
    apiVersion: VISITOR_API_VERSION,
    capabilities: ["devtools-trace"],
  });
  return immutable({
    name: plugin.name,
    version: plugin.version,
    fingerprint: plugin.fingerprint,
    run(deliveryJson) {
      if (typeof deliveryJson !== "string") {
        fail("invalid-event", "delivery", "must be serialized JSON");
      }
      let delivery;
      try {
        delivery = JSON.parse(deliveryJson);
      } catch {
        fail("invalid-event", "delivery", "must be valid JSON");
      }
      record(delivery, "delivery", "invalid-event");
      if (delivery.schemaVersion !== 1) {
        fail("invalid-event", "delivery.schemaVersion", "must equal 1");
      }
      const sessionId = nonempty(delivery.sessionId, "delivery.sessionId", "invalid-event");
      record(delivery.event, "delivery.event", "invalid-event");
      const { event } = delivery;
      if (!Number.isInteger(event.sequence) || event.sequence < 1) {
        fail("invalid-event", "delivery.event.sequence", "must be a positive integer");
      }
      if (
        typeof event.timestamp !== "number" ||
        !Number.isFinite(event.timestamp) ||
        event.timestamp < 0
      ) {
        fail("invalid-event", "delivery.event.timestamp", "must be a non-negative finite number");
      }
      const kind = nonempty(event.kind, "delivery.event.kind", "invalid-event");
      const durationMs = event.durationMs ?? 0;
      if (typeof durationMs !== "number" || !Number.isFinite(durationMs) || durationMs < 0) {
        fail("invalid-event", "delivery.event.durationMs", "must be a non-negative finite number");
      }
      const { sequence, timestamp } = event;
      const startNs = timestamp * 1_000_000;
      const durationNs = durationMs * 1_000_000;
      if (!Number.isFinite(startNs) || !Number.isFinite(durationNs)) {
        fail("invalid-event", "delivery.event", "cannot be represented in nanoseconds");
      }
      const data = Object.fromEntries(
        Object.entries(event).filter(
          ([key]) =>
            key !== "sequence" && key !== "timestamp" && key !== "kind" && key !== "durationMs",
        ),
      );
      const source =
        typeof event.source === "string" && event.source.trim() !== ""
          ? event.source
          : `vize-devtools://${encodeURIComponent(sessionId)}`;
      const separator = kind.indexOf(":");
      runVisitorFixture(plugin, "devtools-trace", {
        apiVersion: VISITOR_API_VERSION,
        capability: "devtools-trace",
        file: source,
        payload: {
          events: [
            {
              id: `${sessionId}:${sequence}`,
              parentId: null,
              name: kind,
              stage: separator === -1 ? kind : kind.slice(0, separator),
              startNs,
              durationNs,
              data,
            },
          ],
        },
      });
      return "";
    },
  });
}

function capabilitiesOf(value, field, requireNonempty) {
  if (!Array.isArray(value) || (requireNonempty && value.length === 0)) {
    fail(
      "invalid-capabilities",
      field,
      requireNonempty ? "must be a non-empty array" : "must be an array",
    );
  }
  const seen = new Set();
  for (const [index, capability] of value.entries()) {
    if (typeof capability !== "string" || !CAPABILITY_ORDER.has(capability)) {
      fail(
        "unknown-capability",
        `${field}[${index}]`,
        `${JSON.stringify(capability)} is not supported`,
      );
    }
    if (seen.has(capability)) {
      fail(
        "duplicate-capability",
        field,
        `${JSON.stringify(capability)} is declared more than once`,
      );
    }
    seen.add(capability);
  }
  return Object.freeze([...seen]);
}

function visitorsOf(value, capabilities) {
  record(value, "visitors", "invalid-visitors");
  const declared = new Set(capabilities);
  for (const capability of Object.keys(value)) {
    if (!CAPABILITY_ORDER.has(capability)) {
      fail(
        "unknown-capability",
        `visitors.${capability}`,
        `${JSON.stringify(capability)} is not supported`,
      );
    }
    if (!declared.has(capability)) {
      fail(
        "undeclared-visitor",
        `visitors.${capability}`,
        "has no matching capability declaration",
      );
    }
  }
  const visitors = {};
  for (const capability of capabilities) {
    const callback = value[capability];
    if (typeof callback !== "function") {
      fail("missing-visitor", `visitors.${capability}`, "must be a synchronous function");
    }
    if (callback.constructor?.name === "AsyncFunction") {
      fail("async-visitor", `visitors.${capability}`, "asynchronous visitors are unsupported");
    }
    visitors[capability] = callback;
  }
  return Object.freeze(visitors);
}

function cacheInputsOf(value) {
  if (!Array.isArray(value)) {
    fail("invalid-cache-inputs", "cacheInputs", "must be an explicit array");
  }
  const inputs = [];
  const names = new Set();
  for (const [index, input] of value.entries()) {
    record(input, `cacheInputs[${index}]`, "invalid-cache-inputs");
    const name = nonempty(input.name, `cacheInputs[${index}].name`, "invalid-cache-inputs");
    if (name.startsWith("@vize/")) {
      fail("invalid-cache-inputs", `cacheInputs[${index}].name`, "uses a reserved host prefix");
    }
    if (names.has(name)) {
      fail(
        "invalid-cache-inputs",
        "cacheInputs",
        `contains duplicate name ${JSON.stringify(name)}`,
      );
    }
    if (typeof input.value !== "string") {
      fail("invalid-cache-inputs", `cacheInputs[${index}].value`, "must be a string");
    }
    names.add(name);
    inputs.push(Object.freeze({ name, value: input.value }));
  }
  return Object.freeze(inputs.sort((left, right) => left.name.localeCompare(right.name)));
}

function apiVersion(value, field) {
  if (value !== VISITOR_API_VERSION) {
    fail(
      "unsupported-api-version",
      field,
      `${JSON.stringify(value)} is unsupported; expected ${VISITOR_API_VERSION}`,
    );
  }
}

function nonempty(value, field, code) {
  if (typeof value !== "string" || value.trim() === "")
    fail(code, field, "must be a non-empty string");
  return value;
}

function record(value, field, code) {
  if (value === null || typeof value !== "object" || Array.isArray(value)) {
    fail(code, field, "must be an object");
  }
}

function fail(code, field, message) {
  throw new VisitorPluginValidationError(code, field, message);
}

function immutable(value) {
  if (value && typeof value === "object" && !Object.isFrozen(value)) {
    for (const child of Object.values(value)) immutable(child);
    Object.freeze(value);
  }
  return value;
}
