export function A({ xs }: { xs: number[] }) @{
	<ul>@for (const x of xs) { if (x) { continue; } <li /> }</ul>
}
