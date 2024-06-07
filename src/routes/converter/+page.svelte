<script lang="ts">
	import { Tab, TabGroup } from '@skeletonlabs/skeleton';
	import { writable } from 'svelte/store';
	import { converterTab } from '$lib/stores';
	import BookUpload from '@tabler/icons-svelte/IconBookUpload.svelte';
	import Books from '@tabler/icons-svelte/IconBooks.svelte';
	import { BundlerFlag } from '$components/converter/types';
	import Scan from '@tabler/icons-svelte/IconScan.svelte';
	import {
		TabConfigureBundler,
		TabSelectBundler,
		TabSourceAnalyze,
		TabSourceSelector,
		TabVolumeSelector,
		TabWrapper
	} from '$components/converter';
	import SettingsAutomation from '@tabler/icons-svelte/IconSettingsAutomation.svelte';

	const classEnabled = 'text-primary-500';
	const classDisabled =
		'text-gray-400 hover:cursor-default hover:!bg-transparent hover:!text-gray-400';

	// stores
	const stores = {
		sourceManga: writable<string | null>(null),
		bundlerRecommendation: writable<BundlerFlag>(BundlerFlag.IMAGE),
		bundler: writable<BundlerFlag | null>(null),
		cpv: writable<Array<number>>([])
	};

	const tabs = [
		{
			name: 'Load Source',
			instruction: 'Please select a directory to use as source material',
			icon: BookUpload,
			component: TabSourceSelector,
			hide: false
		},
		{
			name: 'Analyze Source',
			icon: Scan,
			component: TabSourceAnalyze,
			hide: false
		},
		{
			name: 'Bundling Method',
			instruction: 'Please select the desired volume bundling method',
			component: TabSelectBundler,
			hide: true
		},
		{
			name: 'Volume Bundling',
			icon: Books,
			component: TabVolumeSelector,
			hide: false
		},
		{
			name: 'Configure Converter',
			instruction: 'Configure the converter with your preferred settings',
			icon: SettingsAutomation,
			component: TabConfigureBundler,
			hide: false
		}
	];

	// reset tab when re-entering page
	$converterTab = 0;
</script>

<TabGroup class="w-full">
	{#each tabs as tab, i}
		{#if !tab.hide}
			<Tab
				bind:group={$converterTab}
				name={tab.name ? tab.name : 'Step ' + (i + 1)}
				value={i}
				disabled={$converterTab < i}
				class={$converterTab < i ? classDisabled : $converterTab === i ? classEnabled : ''}
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
				<div class="w-full h-full">
					<TabWrapper title={tab.name} instruction={tab.instruction} index={i} length={tabs.length}>
						<svelte:component this={tab.component} {...stores} />
					</TabWrapper>
				</div>
			{/if}
		{/each}
	</svelte:fragment>
</TabGroup>
