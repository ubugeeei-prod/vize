export interface NativeVueOptionSchema {
  type?: string;
  $ref?: string;
  enum?: readonly unknown[];
  oneOf?: readonly NativeVueOptionSchema[];
  properties?: Readonly<Record<string, NativeVueOptionSchema>>;
  additionalProperties?: boolean | NativeVueOptionSchema;
  items?: NativeVueOptionSchema;
  uniqueItems?: boolean;
  pattern?: string;
  propertyNames?: { pattern?: string };
  required?: readonly string[];
  minimum?: number;
  maximum?: number;
}

/** The exact attribute schema owner retains its legacy scalar-enum diagnostic. */
export function createVueOptionValidator(
  definitions: Readonly<Record<string, NativeVueOptionSchema>>,
  object: (value: unknown) => boolean,
  attributePolicy: NativeVueOptionSchema,
): (value: unknown, schema: NativeVueOptionSchema, label: string) => void;
