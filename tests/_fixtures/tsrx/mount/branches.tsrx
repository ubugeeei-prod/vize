export function Ternary({ a, b }: { a: boolean; b: boolean }) @{
	<p>{a ? (b ? <i>b</i> : <s>not b</s>) : <u>not a</u>}</p>
}

export function Blocks({ a, b }: { a: boolean; b: boolean }) @{
	<p>
		@if (a) {
			@if (b) {
				<i>b</i>
			} @else {
				<s>not b</s>
			}
		}
	</p>
}

export function Labels({ a, b, label }: { a: boolean; b: boolean; label: string }) @{
	<p>
		@if (a) {
			@if (b) {
				<i>{label}</i>
			} @else {
				<s>{label}</s>
			}
		}
	</p>
}

export function Several({ a, n }: { a: boolean; n: number }) @{
	<p>{a ? <><i>{n}</i><s>two</s></> : <u>none</u>}</p>
}

export function Texts({ a, n }: { a: boolean; n: number }) @{
	<p>{a ? <>count {n}</> : null}</p>
}

export function AndIf({ a, n }: { a: boolean; n: number }) @{
	<p>{n && (a ? <i>a</i> : <s>not a</s>)}</p>
}

export function IfAnd({ a, n }: { a: boolean; n: number }) @{
	<p>{a ? (n && <i>{n}</i>) : <u>not a</u>}</p>
}
