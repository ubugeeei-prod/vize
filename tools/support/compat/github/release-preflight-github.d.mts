interface Request {
  apiUrl: string;
  repository: string;
  token: string;
  resource: string;
  method?: string;
  body?: unknown;
  fetchImpl?: typeof fetch;
  requestTimeoutMs?: number;
  sleep?: (milliseconds: number) => Promise<unknown>;
}
export function githubApiRequest(
  options: Request,
): Promise<{ body: string; headers: Headers; status: number }>;
export function githubApiPages(
  options: Request & { collection: string | null; query?: Record<string, string> },
): Promise<Record<string, unknown>[]>;
