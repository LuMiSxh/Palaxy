<script lang="ts">
	import type { Writable } from 'svelte/store';
	import { BundlerFlag } from '$components/converter/types';
	import { ListBox, ListBoxItem } from '@skeletonlabs/skeleton';
	import Prompt from '@tabler/icons-svelte/IconPrompt.svelte';
	import PhotoScan from '@tabler/icons-svelte/IconPhotoScan.svelte';
	import TextScan_2 from '@tabler/icons-svelte/IconTextScan2.svelte';
	import { tabDisableBack, tabDisableNext } from '$lib/stores';

	export let bundler: Writable<BundlerFlag | null>;
	export let bundlerRecommendation: Writable<BundlerFlag>;

	// Initial state
	$tabDisableBack = false;
	$tabDisableNext = true;

	bundler.subscribe(value => {
		if (value !== null) $tabDisableNext = false;
	});
</script>

<div class="w-full min-h-full flex items-center justify-center flex-col">
	<ListBox class="variant-glass-surface h-full mt-2">
		<ListBoxItem
			bind:group={$bundler}
			name="Manual"
			value={BundlerFlag.MANUAL}
			class="px-7 py-3"
		>
			<svelte:fragment slot="lead">
				<Prompt class="text-primary-500" />
			</svelte:fragment>
			Choose volume size manually
		</ListBoxItem>
		<ListBoxItem
			bind:group={$bundler}
			name="Image"
			value={BundlerFlag.IMAGE}
			class="px-7 py-3"
		>
			<svelte:fragment slot="lead">
				<PhotoScan class="text-primary-500" />
			</svelte:fragment>
			Calculate volume size based on cover images
		</ListBoxItem>
		<ListBoxItem
			bind:group={$bundler}
			name="Naming"
			value={BundlerFlag.NAME}
			disabled={BundlerFlag.NAME !== $bundlerRecommendation}
			class="px-7 py-3"
		>
			<svelte:fragment slot="lead">
				<TextScan_2 class="text-primary-500" />
			</svelte:fragment>
			Calculate volume size based on directory naming convention
		</ListBoxItem>
	</ListBox>
</div>