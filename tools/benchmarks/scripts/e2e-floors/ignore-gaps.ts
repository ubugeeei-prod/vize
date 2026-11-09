// Producer ignore tuples ([0, 0, start, end, 0]) cover unmapped gaps; these drop some of them.
type Result = { text: string; diagnosticDirectives?: { directives: number[][] } };
const producer = (row: number[]) => row[0] === 0 && row[1] === 0 && row[4] === 0;
export function dropWhitespaceIgnores(result: Result): number {
  const table = result.diagnosticDirectives; if (!table) return 0;
  const text = Buffer.from(result.text); const before = table.directives.length;
  table.directives = table.directives.filter((row) => !(producer(row) && /^\s*$/.test(text.subarray(row[2], row[3]).toString())));
  return before - table.directives.length;
}
export function dropProducerIgnores(result: Result): number {
  const table = result.diagnosticDirectives; if (!table) return 0;
  const before = table.directives.length;
  table.directives = table.directives.filter((row) => !producer(row));
  return before - table.directives.length;
}
