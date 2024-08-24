<script lang="ts">
	import { ProgressBar } from "@skeletonlabs/skeleton"
	import {
		TabBundleMethodSelector,
		TabBundleSelector,
		TabConfigureConverter,
		TabFilterImages,
		tabIndex,
		tabLoading,
		TabSourceAnalyze,
		TabSourceSelector,
		TabWrapper,
	} from "$components/converter"
	import type { ComponentType } from "svelte"

	interface TabInfo {
		name: string
		instruction?: string
		component: ComponentType
	}

	const tabs: TabInfo[] = [
		{
			name: "Load Source",
			instruction: "Please select a directory to use as source material",
			component: TabSourceSelector,
		},
		{
			name: "Analyze Source",
			instruction: "Analyzing source material for potential issues or improvements",
			component: TabSourceAnalyze,
		},
		{
			name: "Bundling Method",
			instruction: "Please select the desired volume bundling method",
			component: TabBundleMethodSelector,
		},
		{
			name: "Volume Bundling",
			instruction: "Please check and/or adjust the volume sizes",
			component: TabBundleSelector,
		},
		{
			name: "Image Filter",
			instruction: "Please select the images to omit from the conversion",
			component: TabFilterImages,
		},
		{
			name: "Configure Converter",
			instruction: "Configure the converter with your preferred settings",
			component: TabConfigureConverter,
		},
	]

	// reset tab when re-entering page
	$tabIndex = 0
</script>

<div class="flex h-full w-full flex-col">
	<ProgressBar
		value={$tabLoading ? undefined : $tabIndex + 1}
		max={tabs.length}
		meter="bg-gradient-to-r from-primary-500 to-secondary-500"
		rounded="rounded-none"
		class="w-full animate-pulse shadow-lg"
	/>
	{#each tabs as tab, i}
		{#if $tabIndex === i}
			<TabWrapper title={tab.name} instruction={tab.instruction} idx={i} length={tabs.length}>
				<svelte:component this={tab.component} />
			</TabWrapper>
		{/if}
	{/each}
</div>
