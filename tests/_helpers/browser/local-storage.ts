interface StorageSeed {
  values: Record<string, string>;
  clear?: boolean;
}

export function seedLocalStorage({ values, clear }: StorageSeed): void {
  if (clear) localStorage.clear();
  for (const [key, value] of Object.entries(values)) localStorage.setItem(key, value);
}
