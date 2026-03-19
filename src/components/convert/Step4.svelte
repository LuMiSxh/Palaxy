<script lang="ts">
	import { onMount } from 'svelte';
	import { keyHint } from '$states/keyhint.svelte';
	import { t } from 'svelte-i18n-lingui';
	import { wrapper } from '$lib/utils';
	import { step4State } from '$components/convert/step4/utils.svelte';
	import ManualBundling from '$components/convert/step4/ManualBundling.svelte';
	import AutomaticBundling from '$components/convert/step4/AutomaticBundling.svelte';
	import { LoadingSpinner } from 'waku/components';
	import convState from '$states/converter.svelte';
	import { commands, type BundleFlag } from '$types';

	let unregisterKeyHint: () => void;

	onMount(async () => {
		unregisterKeyHint = keyHint.register([['tab', $t`Navigate fields`]]);

		// Sync flatten state to backend before bundling
		await wrapper(commands.convStateSet({ Flatten: convState.flatten }));

		// If flatten is enabled, run bundler immediately (it will merge all into one volume)
		if (convState.flatten) {
			step4State.loading = true;
			const result = await wrapper(commands.convBundle(null));
			step4State.loading = false;

			if (result !== null && result.payload !== null) {
				step4State.result = result.payload;
				convState.chapterSizes = result.payload.chapter_sizes ?? [];
			}
			return;
		}

		// If bundle type already set, run bundler if not manual
		if (convState.bundle && convState.bundle !== 'MANUAL') {
			step4State.loading = true;
			const result = await wrapper(commands.convBundle(step4State.sensibility));
			step4State.loading = false;

			if (result !== null && result.payload !== null) {
				step4State.result = result.payload;
				convState.chapterSizes = result.payload.chapter_sizes ?? [];
			}
		}
	});

	// Save state immediately when values change instead of in onDestroy
	$effect(() => {
		if (convState.bundle) {
			wrapper(commands.convStateSet({ BundleFlag: convState.bundle as BundleFlag }));
		}
	});

	$effect(() => {
		if (convState.chapterSizes.length > 0) {
			wrapper(commands.convStateSet({ VolumeSizes: convState.chapterSizes }));
		}
	});

	onMount(() => {
		return () => {
			if (unregisterKeyHint) unregisterKeyHint();
		};
	});
</script>

{#if step4State.loading}
	<div class="flex h-full w-full items-center justify-center p-3">
		<LoadingSpinner size="lg" text={$t`Running Bundler...`} />
	</div>
{:else if convState.flatten}
	<div class="flex h-full w-full items-center justify-center p-3">
		<div class="max-w-md text-center">
			<div class="bg-accent-500/10 mx-auto mb-4 flex h-16 w-16 items-center justify-center rounded-full">
				<span class="text-3xl">1</span>
			</div>
			<h3 class="mb-2 text-lg font-semibold">{$t`Flatten Mode Active`}</h3>
			<p class="text-muted text-sm">
				{$t`All chapters will be merged into a single volume. You can change this in the Output Format step.`}
			</p>
			{#if step4State.result}
				<p class="text-muted mt-2 text-xs">
					{$t`Total chapters`}: {step4State.result.total_chapters}
				</p>
			{/if}
		</div>
	</div>
{:else if convState.bundle === 'MANUAL'}
	<ManualBundling />
{:else}
	<AutomaticBundling />
{/if}
