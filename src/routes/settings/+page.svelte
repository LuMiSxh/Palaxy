<script lang="ts">
	import { getName, getTauriVersion, getVersion } from '@tauri-apps/api/app';
	import { FeatureFlag, SupportedLanguages, Theme } from '$types/appdata';
	import { convertToTitleCase, ffIsEnabled } from '$lib/utils';
	import { appData } from '$stores/appdata.js';

	const themeEntries = Object.entries(Theme).filter(([key]) => isNaN(Number(key)));
	const langEntries = Object.entries(SupportedLanguages).filter(([key]) => isNaN(Number(key)));

	let featureInput = $state(unParseInput($appData.featureFlags));

	$effect(() => {
		if (featureInput === '') return;
		$appData.featureFlags = parseInput(featureInput);
	});

	function parseInput(input: string): FeatureFlag[] {
		return input
			.split(' ')
			.map(Number)
			.filter((n) => !isNaN(n)) as FeatureFlag[];
	}

	function unParseInput(flags: FeatureFlag[]): string {
		return flags.map(String).join(' ');
	}
</script>

{#snippet label(name: string, value?: string, error?: Error)}
	<div class="glass-surface w-full">
		<fieldset class="fieldset">
			<legend class="fieldset-legend">{name}</legend>
			<input
				type="text"
				readonly
				class={'input invis ' + (error === undefined ? 'input-primary' : 'input-error')}
				value={error ?? value ?? 'None'}
			/>
		</fieldset>
	</div>
{/snippet}

<div class="grid h-full grid-cols-3 grid-rows-3 gap-10 select-none">
	<div class="flex h-full w-full items-center justify-center" style="grid-column: 1; grid-row: 1;">
		{#await getName()}
			{@render label('Name', 'Loading...')}
		{:then name}
			{@render label('Name', name)}
		{:catch error}
			{@render label('Name', undefined, error)}
		{/await}
	</div>
	<div class="flex h-full w-full items-center justify-center" style="grid-column: 1; grid-row: 2;">
		{#await getVersion()}
			{@render label('App Version', 'Loading...')}
		{:then aVersion}
			{@render label('App Version', aVersion)}
		{:catch error}
			{@render label('App Version', undefined, error)}
		{/await}
	</div>
	<div class="flex h-full w-full items-center justify-center" style="grid-column: 1; grid-row: 3;">
		{#await getTauriVersion()}
			{@render label('Tauri Version', 'Loading...')}
		{:then tVersion}
			{@render label('Tauri Version', tVersion)}
		{:catch error}
			{@render label('Tauri Version', undefined, error)}
		{/await}
	</div>
	<div class="flex h-full w-full items-center justify-center" style="grid-column: 2; grid-row: 1;">
		<div class="glass-surface w-full">
			<fieldset class="fieldset">
				<legend class="fieldset-legend">Theme</legend>
				<select id="theme" class="select select-secondary invis w-full" bind:value={$appData.theme}>
					{#each themeEntries as [k, v]}
						<option class="capitalize" value={v}>{convertToTitleCase(k)}</option>
					{/each}
				</select>
			</fieldset>
		</div>
	</div>
	<div class="flex h-full w-full items-center justify-center" style="grid-column: 2; grid-row: 2;">
		<div class="glass-surface w-full">
			<fieldset class="fieldset">
				<legend class="fieldset-legend">Features</legend>
				<input id="feature" class="input input-secondary invis" bind:value={featureInput} />
			</fieldset>
		</div>
	</div>
	{#if ffIsEnabled($appData, FeatureFlag.CHANGE_LANGUAGE)}
		<div
			class="flex h-full w-full items-center justify-center"
			style="grid-column: 2; grid-row: 3;"
		>
			<div class="glass-surface w-full">
				<fieldset class="fieldset">
					<legend class="fieldset-legend">
						Language
						<span class="label-experimental">Experimental</span>
					</legend>
					<select
						id="lang"
						class="select select-secondary invis w-full"
						bind:value={$appData.language}
					>
						{#each langEntries as [k, v]}
							<option class="capitalize" value={v}>{convertToTitleCase(k)}</option>
						{/each}
					</select>
				</fieldset>
			</div>
		</div>
	{/if}
	<!-- TODO: Add autofill for converter (create folder, target path, conversion type (Modal?) -->
</div>
