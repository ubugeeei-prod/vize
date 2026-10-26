/** Append owned parameters without losing existing URL queries or fragments. */
export function styleRequest(
  src: string,
  variables: string,
  scoped: boolean,
  productionFilename?: string,
): string {
  const hash = src.indexOf("#");
  const fragment = hash === -1 ? "" : src.slice(hash);
  const request = hash === -1 ? src : src.slice(0, hash);
  const question = request.indexOf("?");
  // ESM specifiers are URLs. Encode raw Unicode and spaces while preserving
  // existing percent escapes and relative/alias path syntax.
  const file = (question === -1 ? request : request.slice(0, question)).replace(
    /[^\u0021-\u007e]/gu,
    (character) => encodeURIComponent(character),
  );
  const query = new URLSearchParams(question === -1 ? "" : request.slice(question + 1));
  if (scoped) query.set("vize-scope", variables);
  query.set("vize-vars", variables);
  if (productionFilename !== undefined) query.set("vize-owner", productionFilename);
  return `${file}?${query}${fragment}`;
}
