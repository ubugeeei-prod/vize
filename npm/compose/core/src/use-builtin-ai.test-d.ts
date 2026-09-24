/** Compile-only assertions for the `use-builtin-ai` type contracts. */

import { useLanguageDetector, useSummarizer, useTranslator } from "./use-builtin-ai.ts";
import type {
  BuiltinAiAvailability,
  BuiltinAiStatus,
  LanguageDetectionResult,
  TranslatorHost,
} from "./use-builtin-ai.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

const translator = useTranslator({ sourceLanguage: "en", targetLanguage: () => "ja" });
type _Availability = Expect<
  Equal<Awaited<ReturnType<typeof translator.availability>>, BuiltinAiAvailability>
>;
type _Status = Expect<Equal<typeof translator.status.value, BuiltinAiStatus>>;
type _Stream = Expect<
  Equal<ReturnType<typeof translator.translateStreaming>, AsyncIterable<string>>
>;

const detector = useLanguageDetector();
type _Detect = Expect<
  Equal<Awaited<ReturnType<typeof detector.detect>>, readonly LanguageDetectionResult[]>
>;

declare class CustomTranslator {
  static availability(): Promise<"available">;
  static create(): Promise<{
    translate(input: string): Promise<string>;
    translateStreaming(input: string): AsyncIterable<string>;
    destroy(): void;
  }>;
}
CustomTranslator satisfies TranslatorHost;
useTranslator({ sourceLanguage: "en", targetLanguage: "ja", host: CustomTranslator });

// @ts-expect-error the language pair is required.
useTranslator({ sourceLanguage: "en" });

// @ts-expect-error summary types are a closed union.
useSummarizer({ type: "abstract" });

// @ts-expect-error formats are a closed union.
useSummarizer({ format: "html" });

// @ts-expect-error download progress is read-only.
translator.downloadProgress.value = 1;
