export function A({ a, b }: { a: number; b: number }) @{
	const ok = a < b;
	const m = Math.max<number>(a, b);
	<p>{ok ? m : 0}</p>
}
