/** Compile-only assertions for the `use-drop-zone` type contracts. */

import { matchesAccept, useDropZone } from "./use-drop-zone.ts";
import type { DataTransferLike } from "./use-drop-zone.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const element: HTMLElement;
const zone = useDropZone(() => element, {
  accept: ["image/*"],
  onDrop: (files) => {
    type _DroppedFiles = Expect<Equal<typeof files, readonly File[] | null>>;
  },
});
type _FilesAreReadonly = Expect<Equal<typeof zone.files.value, readonly File[] | null>>;

declare const transfer: DataTransfer;
transfer satisfies DataTransferLike;

matchesAccept([".png"], new File([], "a.png")) satisfies boolean;

// @ts-expect-error accept patterns are strings or a predicate.
useDropZone(() => element, { accept: [1] });

// @ts-expect-error the drag state is read-only.
zone.isOverDropZone.value = true;
