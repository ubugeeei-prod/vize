import type { AdmissionMutation } from "./type-admission-fixtures.ts";

/** UTF-16 source coordinate, independently derived from the authored mutation. */
export function mutatedAdmission(source: string, mutation: AdmissionMutation) {
  if (source.split(mutation.before).length !== 2) throw new Error("mutation anchor must be unique");
  const offset = source.indexOf(mutation.before);
  const mutated = source.replace(mutation.before, mutation.after);
  const tokenOffset = mutation.after.indexOf(mutation.token);
  if (tokenOffset < 0) throw new Error("diagnostic token is missing from mutation");
  const prefix = mutated.slice(0, offset + tokenOffset).split("\n");
  return {
    source: mutated,
    line: prefix.length,
    column: prefix.at(-1)!.length + 1,
    code: mutation.code,
  };
}
