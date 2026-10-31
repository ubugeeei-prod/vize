// Product config runtime: plain ESM syntax also embeds unchanged in Node^22 modules.
export function createVueOptionValidator(definitions, object, attributePolicy) {
  function validateVueOption(value, schema, label) {
    if (schema === attributePolicy && typeof value === "string") {
      return validateVueOption(value, schema.oneOf[0], label);
    }
    const refuse = (reason) => {
      throw new TypeError(`${label}: ${reason}`);
    };
    if (schema.$ref) {
      return validateVueOption(value, definitions[schema.$ref.split("/").at(-1)], label);
    }
    if (schema.oneOf) {
      const matches = schema.oneOf.filter((candidate) => {
        try {
          validateVueOption(value, candidate, label);
          return true;
        } catch (error) {
          if (!(error instanceof TypeError)) throw error;
          return false;
        }
      });
      if (matches.length !== 1) refuse("must match one documented native option type");
      return;
    }
    if (schema.enum && !schema.enum.includes(value)) refuse("unsupported native option value");
    const valid = {
      object: object(value),
      array: Array.isArray(value),
      string: typeof value === "string",
      boolean: typeof value === "boolean",
      integer: Number.isInteger(value),
      number: typeof value === "number" && Number.isFinite(value),
      null: value === null,
    };
    if (schema.type && !valid[schema.type]) refuse(`must be ${schema.type}`);
    if (schema.minimum !== undefined && value < schema.minimum)
      refuse(`must be >= ${schema.minimum}`);
    if (schema.maximum !== undefined && value > schema.maximum)
      refuse(`must be <= ${schema.maximum}`);
    if (typeof value === "string" && schema.pattern && !new RegExp(schema.pattern).test(value))
      refuse(`must match ${schema.pattern}`);
    if (Array.isArray(value) && schema.items) {
      for (let index = 0; index < value.length; index++) {
        validateVueOption(value[index], schema.items, `${label}[${index}]`);
      }
    }
    if (Array.isArray(value) && schema.uniqueItems) {
      const keys = value.map((item) =>
        JSON.stringify(item, (_key, entry) =>
          object(entry)
            ? Object.fromEntries(
                Object.entries(entry).sort(([a], [b]) => (a < b ? -1 : a > b ? 1 : 0)),
              )
            : entry,
        ),
      );
      if (new Set(keys).size !== keys.length) refuse("must contain unique items");
    }
    if (object(value)) {
      for (const required of schema.required ?? []) {
        if (!Object.hasOwn(value, required) || value[required] === undefined)
          refuse(`missing ${required}`);
      }
      for (const [key, item] of Object.entries(value)) {
        const documented = Object.hasOwn(schema.properties ?? {}, key);
        if (!documented && schema.additionalProperties === false) refuse(`unknown option "${key}"`);
        if (schema.propertyNames?.pattern && !new RegExp(schema.propertyNames.pattern).test(key)) {
          refuse(`invalid native option name ${key}`);
        }
        const rule = documented ? schema.properties[key] : schema.additionalProperties;
        if (item !== undefined && object(rule)) validateVueOption(item, rule, `${label}.${key}`);
      }
    }
  }
  return validateVueOption;
}
