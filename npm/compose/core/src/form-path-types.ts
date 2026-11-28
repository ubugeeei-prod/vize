/**
 * Path and error types for {@link useForm}.
 *
 * Paths are template-literal keys, bounded at eight levels so type checking
 * stays fast. Leaves stop the walk.
 */

/** Values treated as leaves: paths never descend into them. */
export type FormLeaf =
  | string
  | number
  | bigint
  | boolean
  | symbol
  | null
  | undefined
  | Date
  | RegExp
  | Blob
  | Map<unknown, unknown>
  | Set<unknown>
  | ((...arguments_: never[]) => unknown);

type Decrement = [never, 0, 1, 2, 3, 4, 5, 6, 7, 8];

/**
 * Every dotted path into `Values`, as template-literal keys.
 *
 * Objects contribute their string keys, arrays and tuples contribute
 * `${number}` segments, and {@link FormLeaf} values end a path. Recursion
 * is bounded at eight levels to keep type checking fast.
 */
export type FormPath<Values, Depth extends number = 8> = [Depth] extends [never]
  ? never
  : Values extends FormLeaf
    ? never
    : Values extends readonly (infer Item)[]
      ? `${number}` | `${number}.${FormPath<Item, Decrement[Depth]>}`
      : {
          [Key in keyof Values & string]:
            | Key
            | (FormPath<Values[Key], Decrement[Depth]> extends infer Rest extends string
                ? [Rest] extends [never]
                  ? never
                  : `${Key}.${Rest}`
                : never);
        }[keyof Values & string];

/** Value type found at `Path` inside `Values`. */
export type FormPathValue<Values, Path extends string> = Path extends `${infer Head}.${infer Rest}`
  ? Values extends readonly (infer Item)[]
    ? Head extends `${number}`
      ? FormPathValue<Item, Rest>
      : never
    : Head extends keyof Values
      ? FormPathValue<Values[Head], Rest>
      : never
  : Values extends readonly (infer Item)[]
    ? Path extends `${number}`
      ? Item
      : never
    : Path extends keyof Values
      ? Values[Path]
      : never;

/** Paths whose value is an array (usable with field arrays). */
export type FormArrayPath<Values> = {
  [Path in FormPath<Values>]: FormPathValue<Values, Path> extends readonly unknown[] ? Path : never;
}[FormPath<Values>];

/** Item type of the array at `Path`. */
export type FormArrayItem<Values, Path extends string> =
  FormPathValue<Values, Path> extends readonly (infer Item)[] ? Item : never;

/**
 * Error messages keyed by dotted path; `""` holds form-level errors.
 *
 * Known paths are typed for autocompletion; the string index signature
 * stays because schemas and servers may report paths that are not fields.
 */
export type FormErrors<Values> = {
  readonly [Path in FormPath<Values> | ""]?: readonly string[];
} & { readonly [path: string]: readonly string[] | undefined };

/** Current errors as recorded: messages keyed by dotted path (`""` = form level). */
export type FormErrorRecord = Readonly<Record<string, readonly string[]>>;

/** Result of a validator: messages, one message, or nothing when valid. */
export type ValidatorResult = string | readonly string[] | null | undefined | void;

/** Context passed to field validators. */
export interface FieldValidatorContext<Values> {
  /** Current form values (`undefined` for standalone fields). */
  readonly values: Values;

  /** Dotted path of the validated field. */
  readonly path: string;

  /** Aborted when a newer validation supersedes this one. */
  readonly signal: AbortSignal;
}

/** Plain validator for one field value. */
export type FieldValidator<Value, Values = undefined> = (
  value: Value,
  context: FieldValidatorContext<Values>,
) => ValidatorResult | Promise<ValidatorResult>;

/** Plain validator for the whole form, returning errors keyed by path. */
export type FormValidator<Values> = (
  values: Values,
  context: { readonly signal: AbortSignal },
) => FormErrors<Values> | undefined | void | Promise<FormErrors<Values> | undefined | void>;

/** Per-field validators keyed by path. */
export type FormFieldValidators<Values> = {
  readonly [Path in FormPath<Values>]?: FieldValidator<FormPathValue<Values, Path>, Values>;
};

/** One dirty branch: `true` for a leaf, or a nested map when children differ. */
export type DirtyNode<Values> = true | DirtyFields<Values>;

/**
 * Nested dirty flags against the initial values.
 *
 * A dirty leaf is `true`. Clean branches are omitted. Array indexes stay
 * aligned with the value, and a clean item is `undefined`.
 */
export type DirtyFields<Values> = Values extends FormLeaf
  ? true
  : Values extends readonly (infer Item)[]
    ? readonly (DirtyNode<Item> | undefined)[]
    : Values extends object
      ? { readonly [Key in keyof Values]?: DirtyNode<Values[Key]> }
      : true;
