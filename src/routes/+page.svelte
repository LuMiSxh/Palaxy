<script lang="ts">
	import { IconSearch, IconSettings, IconTransform, IconUsers } from '@tabler/icons-svelte';
	import { t } from 'svelte-i18n-lingui';
	import { goto } from '$app/navigation';
	import { ffIsEnabled } from '$lib/utils';
	import { appData } from '$stores/appdata';
	import { FeatureFlag } from '$types/appdata';
</script>

<div class="grid h-full w-full grid-cols-2 grid-rows-2 gap-10 p-3 select-none">
	<div
		role="button"
		onkeydown={() => null}
		tabindex="0"
		onclick={() => goto('/convert')}
		class="glass-surface relative flex h-full w-full cursor-pointer flex-col overflow-hidden transition-transform hover:scale-[102%] active:scale-[98%]"
		style="grid-column: 1; grid-row: 1;"
	>
		<div class="z-20 mt-3 ml-3">
			<h1 class="text-2xl font-bold select-none">{$t`Convert`}</h1>
			<p class="mt-6 mr-3 select-none">
				<span class="label-stable">{$t`Stable`}</span><br />
				{$t`Convert your manga images into a digital format that can be read on your favorite devices.`}
			</p>
		</div>
		<IconTransform size="60" class="absolute top-3 right-3" />
	</div>
	{#if ffIsEnabled($appData, FeatureFlag.SEARCH_MANGA)}
		<div
			role="button"
			onkeydown={() => null}
			tabindex="0"
			onclick={() => goto('/search')}
			class="glass-surface relative flex h-full w-full cursor-pointer flex-col overflow-hidden transition-transform hover:scale-[102%] active:scale-[98%]"
			style="grid-column: 1; grid-row: 2;"
		>
			<div class="z-20 mt-3 ml-3">
				<h1 class="text-2xl font-bold select-none">{$t`Search`}</h1>
				<p class="mt-6 mr-3 select-none">
					<span class="label-experimental">{$t`Experimental`}</span><br />
					{$t`Search for your favorite manga series and chapters from various sources.`}
				</p>
			</div>
			<IconSearch size="60" class="absolute top-3 right-3" />
		</div>
	{/if}
	{#if ffIsEnabled($appData, FeatureFlag.BROWSE_AGENTS)}
		<div
			role="button"
			onkeydown={() => null}
			tabindex="0"
			onclick={() => goto('/agents')}
			class="glass-surface relative flex h-full w-full cursor-pointer flex-col overflow-hidden transition-transform hover:scale-[102%] active:scale-[98%]"
			style="grid-column: 2; grid-row: 2;"
		>
			<div class="z-20 mt-3 ml-3">
				<h1 class="text-2xl font-bold select-none">{$t`Agents`}</h1>
				<p class="mt-6 mr-3 select-none">
					<span class="label-experimental">{$t`Experimental`}</span><br />
					{$t`Manage your agents and their settings for better search results.`}
				</p>
			</div>
			<IconUsers size="60" class="absolute top-3 right-3" />
		</div>
	{/if}
	<div
		role="button"
		onkeydown={() => null}
		tabindex="0"
		onclick={() => goto('/settings')}
		class="glass-surface relative flex h-full w-full cursor-pointer flex-col overflow-hidden transition-transform hover:scale-[102%] active:scale-[98%]"
		style="grid-column: 2; grid-row: 1;"
	>
		<div class="z-20 mt-3 ml-3">
			<h1 class="text-2xl font-bold select-none">{$t`Settings`}</h1>
			<p class="mt-6 mr-3 select-none">
				{$t`Change Palaxy's settings to better suit your needs and preferences.`}
			</p>
		</div>
		<IconSettings size="60" class="absolute top-3 right-3" />
	</div>
</div>
