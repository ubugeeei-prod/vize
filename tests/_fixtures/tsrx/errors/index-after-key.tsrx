export function A({ xs }: { xs: number[] }) @{
	<ul>@for (const x of xs; key x; index i) { <li /> }</ul>
}
