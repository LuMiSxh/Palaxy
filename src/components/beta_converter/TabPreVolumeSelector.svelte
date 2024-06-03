<script lang="ts">
	import { converterTab } from '$lib/stores';
	import type { Writable } from 'svelte/store';
	import { BundlerFlag } from '$components/beta_converter/types';
	import { ListBox, ListBoxItem } from '@skeletonlabs/skeleton';
	import Prompt from '@tabler/icons-svelte/IconPrompt.svelte';
	import PhotoScan from '@tabler/icons-svelte/IconPhotoScan.svelte';
	import TextScan_2 from '@tabler/icons-svelte/IconTextScan2.svelte';

	export let bundler: Writable<BundlerFlag | null>;
	export let bundlerRecommendation: Writable<BundlerFlag>;
</script>

<div class="w-full min-h-full flex items-center justify-center flex-col">
	<h4 class="h4">Please select a bundling method.</h4>
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
	<div class="w-full flex justify-evenly pt-4">
		<button
			on:click={() => converterTab.update(tab => tab-1)}
			disabled={false}
			class="btn variant-ghost-secondary btn-lg"
		>
			Go Back
		</button>
		<button
			disabled={$bundler === null}
			on:click={() => converterTab.update(tab => tab+1)}
			class="btn variant-ghost-primary btn-lg"
		>
			Next Step
		</button>
	</div>
</div>