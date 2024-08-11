<script lang="ts">
	import { Tab, TabGroup } from '@skeletonlabs/skeleton';
	import { writable } from 'svelte/store';
	import { converterTab } from '$lib/stores';
	import { BundlerFlag } from '$components/converter/types';
	import {
		TabConfigureConverter,
		TabSourceAnalyze,
		TabSourceSelector,
		TabVolumeSelector,
		TabVolumeSorter,
		TabWrapper
	} from '$components/converter';
	import type { ComponentType } from 'svelte';
	import { createFSMStore, type Transition, type TransitionValue } from '$lib/fsm';
	import {
		IconBooks,
		IconBookUpload,
		IconScan,
		IconSettingsAutomation
	} from '@tabler/icons-svelte';

	const classEnabled = 'text-primary-500';
	const classDisabled =
		'text-gray-400 hover:cursor-default hover:!bg-transparent hover:!text-gray-400';

	// FSM
	const transitions: Transition<string>[] = [
		{
			From: 'Load Source',
			Via: 'Source Selected',
			To: 'Analyze Source'
		},
		{
			From: 'Analyze Source',
			Via: 'Analyzed',
			To: 'Bundling Method'
		},
		{
			From: 'Bundling Method',
			Via: 'Method Selected',
			To: 'Volume Bundling'
		},
		{
			From: 'Volume Bundling',
			Via: 'Volumes Bundled',
			To: 'Configure Converter'
		}
	];

	const transitionValues: TransitionValue<string, number>[] = [
		{
			Transition: 'Load Source',
			Value: 1
		},
		{
			Transition: 'Analyze Source',
			Value: 2
		},
		{
			Transition: 'Bundling Method',
			Value: 3
		},
		{
			Transition: 'Volume Bundling',
			Value: 4
		}
	];

	// stores
	const stores = {
		sourceManga: writable<string | null>(null),
		bundlerRecommendation: writable<BundlerFlag>(BundlerFlag.IMAGE),
		bundler: writable<BundlerFlag | null>(null),
		cpv: writable<Array<number>>([]),
		fsm: createFSMStore(transitions, transitionValues)
	};

	interface TabInfo {
		name: string;
		instruction?: string;
		icon?: ComponentType;
		component: ComponentType;
		hide: boolean;
	}

	const tabs: TabInfo[] = [
		{
			name: 'Load Source',
			instruction: 'Please select a directory to use as source material',
			icon: IconBookUpload,
			component: TabSourceSelector,
			hide: false
		},
		{
			name: 'Analyze Source',
			icon: IconScan,
			component: TabSourceAnalyze,
			hide: false
		},
		{
			name: 'Bundling Method',
			instruction: 'Please select the desired volume bundling method',
			component: TabVolumeSelector,
			hide: true
		},
		{
			name: 'Volume Bundling',
			icon: IconBooks,
			component: TabVolumeSorter,
			hide: false
		},
		{
			name: 'Configure Converter',
			instruction: 'Configure the converter with your preferred settings',
			icon: IconSettingsAutomation,
			component: TabConfigureConverter,
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
