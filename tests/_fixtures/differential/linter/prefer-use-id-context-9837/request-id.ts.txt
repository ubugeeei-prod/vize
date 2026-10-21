export function createRequestId(): string {
  return crypto.randomUUID();
}
