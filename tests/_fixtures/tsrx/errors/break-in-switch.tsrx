export function Broken({ mode }: { mode: number }) @{
	@switch (mode) {
		@case 1: {
			break;
		}
	}
}
