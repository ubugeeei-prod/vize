export function A({ a }: { a: boolean }) @{
	<div>@if (a) { <b /> } else { <i /> }</div>
}
