export function A({ m }: { m: number }) @{
	@switch (m) {
		@case 1:
		@case 2: {
			<b />
		}
	}
}
