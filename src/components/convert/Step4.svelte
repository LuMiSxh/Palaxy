<script lang="ts">
	import { onDestroy, onMount } from 'svelte';
	import { keyHint } from '$states/keyhint.svelte';
	import { t } from 'svelte-i18n-lingui';
	import { wrapper } from '$lib/utils';
	import { step4State } from '$components/convert/step4/utils.svelte';
	import ManualBundling from '$components/convert/step4/ManualBundling.svelte';
	import AutomaticBundling from '$components/convert/step4/AutomaticBundling.svelte';
	import { VStack } from 'waku/layout';
	import { LoadingSpinner } from 'waku/components';
	import convState from '$states/converter.svelte';
	import { commands, type BundleFlag } from '$types';

	let unregisterKeyHint: () => void;

	onMount(async () => {
		unregisterKeyHint = keyHint.register([['tab', $t`Navigate fields`]]);

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

	onDestroy(async () => {
		if (unregisterKeyHint) unregisterKeyHint();

		// Save chapter sizes and bundle type
		await wrapper(commands.convStateSet({ BundleFlag: convState.bundle as BundleFlag }));
		await wrapper(commands.convStateSet({ VolumeSizes: convState.chapterSizes }));
	});
</script>

{#if step4State.loading}
	<div class="flex h-full w-full items-center justify-center p-3">
		<VStack gap="sm" align="center">
			<LoadingSpinner size="lg" />
			<span class="text-muted text-sm">{$t`Running Bundler...`}</span>
		</VStack>
	</div>
{:else if convState.bundle === 'MANUAL'}
	<ManualBundling />
{:else}
	<AutomaticBundling />
{/if}
