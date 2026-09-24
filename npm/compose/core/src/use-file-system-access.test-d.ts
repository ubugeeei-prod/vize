/** Compile-only assertions for the `use-file-system-access` type contracts. */

import { useFileSystemAccess } from "./use-file-system-access.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

const text = useFileSystemAccess();
type _TextByDefault = Expect<Equal<typeof text.data.value, string | undefined>>;

const bytes = useFileSystemAccess({ dataType: "arrayBuffer" });
type _BytesFromDataType = Expect<Equal<typeof bytes.data.value, ArrayBuffer | undefined>>;

const blob = useFileSystemAccess({ dataType: "blob" });
type _BlobFromDataType = Expect<Equal<typeof blob.data.value, Blob | undefined>>;

// @ts-expect-error a non-text representation must come from a runtime dataType.
useFileSystemAccess<"blob">({});

// @ts-expect-error data keeps the inferred representation.
text.data.value = new ArrayBuffer(1);

// @ts-expect-error unknown data types are rejected.
useFileSystemAccess({ dataType: "json" });
