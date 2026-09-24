/**
 * Minimal JSON Schema (2020-12 subset) validator used by the registry tests.
 *
 * Supports exactly the keywords `vize-lib-registry.schema.json` uses, and
 * reports any other keyword as an error so the schema cannot silently grow
 * constraints the tests would ignore.
 */

type Json = null | boolean | number | string | readonly Json[] | { readonly [key: string]: Json };

const supportedKeywords = new Set([
  "$schema",
  "$id",
  "$defs",
  "$ref",
  "title",
  "description",
  "type",
  "const",
  "enum",
  "pattern",
  "minLength",
  "minimum",
  "minItems",
  "required",
  "additionalProperties",
  "properties",
  "items",
]);

function isObject(value: Json | undefined): value is { readonly [key: string]: Json } {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function typeMatches(type: Json, value: Json): boolean {
  switch (type) {
    case "object":
      return isObject(value);
    case "array":
      return Array.isArray(value);
    case "string":
      return typeof value === "string";
    case "integer":
      return typeof value === "number" && Number.isInteger(value);
    case "number":
      return typeof value === "number";
    case "boolean":
      return typeof value === "boolean";
    default:
      return false;
  }
}

function resolveRef(root: Json, ref: string): Json {
  if (!ref.startsWith("#/")) throw new Error(`unsupported $ref ${ref}`);
  let current: Json | undefined = root;
  for (const segment of ref.slice(2).split("/")) {
    current = isObject(current) ? current[segment] : undefined;
  }
  if (current === undefined) throw new Error(`unresolved $ref ${ref}`);
  return current;
}

function validateNode(root: Json, schema: Json, value: Json, at: string, errors: string[]): void {
  if (!isObject(schema)) return;
  for (const keyword of Object.keys(schema)) {
    if (!supportedKeywords.has(keyword))
      errors.push(`${at}: unsupported schema keyword ${keyword}`);
  }
  const ref = schema["$ref"];
  if (typeof ref === "string") validateNode(root, resolveRef(root, ref), value, at, errors);
  const type = schema["type"];
  if (type !== undefined && !typeMatches(type, value)) {
    errors.push(`${at}: expected ${JSON.stringify(type)}`);
    return;
  }
  if ("const" in schema && JSON.stringify(schema["const"]) !== JSON.stringify(value)) {
    errors.push(`${at}: expected const ${JSON.stringify(schema["const"])}`);
  }
  const enumValues = schema["enum"];
  if (Array.isArray(enumValues) && !enumValues.some((candidate) => candidate === value)) {
    errors.push(`${at}: ${JSON.stringify(value)} is not one of ${JSON.stringify(enumValues)}`);
  }
  if (typeof value === "string") {
    const pattern = schema["pattern"];
    if (typeof pattern === "string" && !new RegExp(pattern, "u").test(value)) {
      errors.push(`${at}: ${JSON.stringify(value)} does not match ${pattern}`);
    }
    const minLength = schema["minLength"];
    if (typeof minLength === "number" && value.length < minLength) errors.push(`${at}: too short`);
  }
  const minimum = schema["minimum"];
  if (typeof value === "number" && typeof minimum === "number" && value < minimum) {
    errors.push(`${at}: below minimum ${minimum}`);
  }
  if (Array.isArray(value)) {
    const minItems = schema["minItems"];
    if (typeof minItems === "number" && value.length < minItems)
      errors.push(`${at}: too few items`);
    const items = schema["items"];
    if (items !== undefined) {
      value.forEach((item, index) => validateNode(root, items, item, `${at}[${index}]`, errors));
    }
  }
  if (isObject(value)) {
    const required = schema["required"];
    if (Array.isArray(required)) {
      for (const key of required) {
        if (typeof key === "string" && !(key in value)) errors.push(`${at}: missing ${key}`);
      }
    }
    const properties = isObject(schema["properties"]) ? schema["properties"] : {};
    for (const [key, child] of Object.entries(value)) {
      const childSchema = properties[key];
      if (childSchema !== undefined) validateNode(root, childSchema, child, `${at}.${key}`, errors);
      else if (schema["additionalProperties"] === false) errors.push(`${at}: unexpected ${key}`);
    }
  }
}

/** Validate `value` against `schema`; returns human-readable errors (empty when valid). */
export function validateJsonSchemaSubset(schema: unknown, value: unknown): readonly string[] {
  const errors: string[] = [];
  const rootSchema = JSON.parse(JSON.stringify(schema)) as Json;
  const rootValue = JSON.parse(JSON.stringify(value)) as Json;
  validateNode(rootSchema, rootSchema, rootValue, "$", errors);
  return errors;
}
