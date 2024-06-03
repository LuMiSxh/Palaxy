<script lang="ts">
	import { Tab, TabGroup } from '@skeletonlabs/skeleton';
	import { writable } from 'svelte/store';
	import { converterTab } from '$lib/stores';
	import Placeholder from '@tabler/icons-svelte/IconPlaceholder.svelte';
	import BookUpload from '@tabler/icons-svelte/IconBookUpload.svelte';
	import TabDirectories from '$components/beta_converter/TabDirectories.svelte';
	import TabAnalyzeSource from '$components/beta_converter/TabAnalyzeSource.svelte';
	import TabPreVolumeSelector from '$components/beta_converter/TabPreVolumeSelector.svelte';
	import Books from '@tabler/icons-svelte/IconBooks.svelte';
	import { BundlerFlag } from '$components/beta_converter/types';
	import Scan from '@tabler/icons-svelte/IconScan.svelte';
	import TabVolumeSelector from '$components/beta_converter/TabVolumeSelector.svelte';

	const classEnabled = 'text-primary-500';
	const classDisabled = 'text-gray-400 hover:cursor-default hover:!bg-transparent hover:!text-gray-400';

	const tabs = [
		{ name: 'Load Source', icon: BookUpload, component: TabDirectories, hide: false },
		{ name: 'Analyze Source', icon: Scan, component: TabAnalyzeSource, hide: false },
		{ name: 'Placeholder', icon: Placeholder, component: TabPreVolumeSelector, hide: true },
		{ name: 'Volume Bundling', icon: Books, component: TabVolumeSelector, hide: false }
	];

	// stores
	const stores = {
		sourceManga: writable<string | null>(null),
		bundlerRecommendation: writable<BundlerFlag>(BundlerFlag.IMAGE),
		bundler: writable<BundlerFlag | null>(null),
	};

	// Pipeline
	// TODO: Write out the pipeline steps out under this comment
</script>

<TabGroup class="w-full">
	{#each tabs as tab, i}
		{#if !tab.hide}
			<Tab
				bind:group={$converterTab}
				name={tab.name}
				value={i}
				disabled={$converterTab < i}
				class={$converterTab < i? classDisabled : $converterTab === i? classEnabled : ""}
			>
				<div slot="lead" class="flex items-center justify-center">
					<svelte:component this={tab.icon} />
				</div>
				{tab.name}
			</Tab>
		{/if}
	{/each}
	<!-- Tab Panels --->
	<svelte:fragment slot="panel">
		{#each tabs as tab, i}
			{#if $converterTab === i}
				<div class="p-4 m-4 min-h-[60vh] flex items-center justify-center card">
					<svelte:component this={tab.component} {...stores} />
				</div>
			{/if}
		{/each}
	</svelte:fragment>
</TabGroup>

<!-- DEBUG -->
<button class="btn variant-ghost-primary" on:click={() => converterTab.update(n => n+1)}>Increase tab counter</button>
<button class="btn variant-ghost-secondary" on:click={() => converterTab.update(n => n-1)}>Decrease tab counter</button>
