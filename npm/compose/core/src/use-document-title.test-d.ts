/** Compile-only assertions for the `use-document-title` type contracts. */

import type { Ref } from "vue";

import { useDocumentTitle } from "./use-document-title.ts";
import type { DocumentTitleHost } from "./use-document-title.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

const { title } = useDocumentTitle("Home", { template: "%s | Vize" });
type _TitleIsNullableString = Expect<Equal<typeof title, Ref<string | null | undefined>>>;

declare const document: Document;
document satisfies DocumentTitleHost;

useDocumentTitle(undefined, { restoreOnDispose: (original) => original });

// @ts-expect-error titles are strings.
useDocumentTitle(1);

// @ts-expect-error the restore policy is a closed union.
useDocumentTitle("x", { restoreOnDispose: "initial" });
