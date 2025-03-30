<script lang="ts">
	import { t } from 'svelte-i18n-lingui';

	let { text = $bindable($t`Loading...`) }: { text: string } = $props();
</script>

<div class="flex flex-col items-center justify-center py-8">
	<div class="mb-4">
		<svg
			xmlns="http://www.w3.org/2000/svg"
			width="96"
			height="96"
			viewBox="0 0 24 24"
			fill="none"
			stroke-width="2"
			stroke-linecap="round"
			stroke-linejoin="round"
		>
			<path stroke="none" d="M0 0h24v24H0z" fill="none" />
			<path d="M3 12a9 9 0 0 0 9 9a9 9 0 0 0 9 -9a9 9 0 0 0 -9 -9" class="outer-circle" />
			<path d="M17 12a5 5 0 1 0 -5 5" class="inner-circle" />
		</svg>
	</div>
	<p class="text-content-secondary dark:text-content-dark-secondary">
		{text}
	</p>
</div>

<style>
	.outer-circle {
		stroke: var(--color-primary);
		transform-origin: center;
		animation: spin-clockwise 2s infinite linear;
	}

	.inner-circle {
		stroke: var(--color-secondary);
		transform-origin: center;
		animation: spin-counterclockwise 1.5s infinite linear;
	}

	@keyframes spin-clockwise {
		0% {
			transform: rotate(0deg);
		}
		100% {
			transform: rotate(360deg);
		}
	}

	@keyframes spin-counterclockwise {
		0% {
			transform: rotate(0deg);
		}
		100% {
			transform: rotate(-360deg);
		}
	}
</style>
