type Options = Record<string, unknown>;
export type Qs = {
  parse(input: string, options?: Options): unknown;
  stringify(input: unknown, options?: Options): string;
};
export type Case = { name: string; make: (qs: Qs) => unknown; options: Options };

export const negative: Case[] = [
  {
    name: "original-allowPrototypes",
    make: (q) => q.parse("a[0][toString]=x", { allowPrototypes: true }),
    options: { arrayFormat: "comma" },
  },
  {
    name: "original-json",
    make: () => JSON.parse('{"a":[{"toString":"x"}]}'),
    options: { arrayFormat: "comma" },
  },
  {
    name: "original-encodeValuesOnly",
    make: () => JSON.parse('{"a":[{"toString":"x"}]}'),
    options: { arrayFormat: "comma", encodeValuesOnly: true },
  },
  {
    name: "null-prototype",
    make: (q) => q.parse("a[][b]=c", { plainObjects: true }),
    options: { arrayFormat: "comma" },
  },
  {
    name: "null-prototype-encoded",
    make: (q) => q.parse("a[][b]=c", { plainObjects: true }),
    options: { arrayFormat: "comma", encodeValuesOnly: true },
  },
  {
    name: "nested-json",
    make: () => JSON.parse('{"a":[[{"toString":"x"}]]}'),
    options: { arrayFormat: "comma" },
  },
  {
    name: "nested-json-encoded",
    make: () => JSON.parse('{"a":[[{"toString":"x"}]]}'),
    options: { arrayFormat: "comma", encodeValuesOnly: true },
  },
];
export const controls: Case[] = [
  { name: "ordinary-comma", make: () => ({ a: ["x", "y"] }), options: { arrayFormat: "comma" } },
  {
    name: "ordinary-encoded",
    make: () => ({ a: ["x", "y"] }),
    options: { arrayFormat: "comma", encodeValuesOnly: true },
  },
  { name: "ordinary-object", make: () => ({ a: [{ b: 1 }] }), options: { arrayFormat: "comma" } },
  { name: "default-array-format", make: () => JSON.parse('{"a":[{"toString":"x"}]}'), options: {} },
  {
    name: "callable-toString",
    make: () => ({ a: [{ toString: () => "z" }] }),
    options: { arrayFormat: "comma" },
  },
  {
    name: "primitive-valueOf",
    make: () => ({ a: [{ toString: "x", valueOf: () => 7 }] }),
    options: { arrayFormat: "comma" },
  },
  {
    name: "null-and-undefined",
    make: () => ({ a: [null, undefined, "x"] }),
    options: { arrayFormat: "comma", encodeValuesOnly: true },
  },
  {
    name: "nested-values",
    make: () => ({ a: [[1, 2], 3] }),
    options: { arrayFormat: "comma", encode: false },
  },
  {
    name: "symbol-toPrimitive",
    make: () => ({ a: [{ toString: "x", [Symbol.toPrimitive]: () => "p" }] }),
    options: { arrayFormat: "comma" },
  },
  {
    name: "authored-throw-propagates",
    make: () => ({
      a: [
        {
          toString: () => {
            throw new SyntaxError("authored-error");
          },
        },
      ],
    }),
    options: { arrayFormat: "comma" },
  },
];
