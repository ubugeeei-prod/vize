export function A({ x }: { x: boolean }) @{
	<div>@{ if (x) { return null; } <p /> }</div>
}
