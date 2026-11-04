import { type Transform } from "./content-mapper-instance-globals-10617.ts";

/** #11675 gives generated wrapping parentheses the ordinary token's diagnostic owner. */
export function parenthesisDiagnosticMappings(result: Pick<Transform, "text" | "mappings">) {
  const bytes = Buffer.from(result.text);
  // Delimiter rows cannot certify their own owner or hide a malformed delimiter.
  const owners = result.mappings.filter(
    ([at, length]) => !(length === 1 && (bytes[at] === 0x28 || bytes[at] === 0x29)),
  );
  return owners.flatMap((span, index) => {
    if (span[5] !== 1048575) return [];
    const [start, length, original, originalLength] = span;
    const open = start - 1;
    const close = start + length;
    const previousEnd = index === 0 ? 0 : owners[index - 1][0] + owners[index - 1][1];
    const nextStart = index + 1 === owners.length ? bytes.length : owners[index + 1][0];
    if (bytes[open] !== 0x28 || bytes[close] !== 0x29 || previousEnd > open || close >= nextStart)
      return [];
    return [
      [open, 1, original, originalLength, 1, 0],
      [close, 1, original, originalLength, 1, 0],
    ];
  });
}
