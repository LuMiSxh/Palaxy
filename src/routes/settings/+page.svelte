<script lang="ts">
	import { getName, getTauriVersion, getVersion } from '@tauri-apps/api/app';
	import { FeatureFlag, SupportedLanguages, Theme } from '$types/appdata';
	import { convertToTitleCase, ffIsEnabled } from '$lib/utils';
	import { appData } from '$stores/appdata.js';

	const themeEntries = Object.entries(Theme).filter(([key, _]) => isNaN(Number(key)));
	const langEntries = Object.entries(SupportedLanguages).filter(([key, _]) => isNaN(Number(key)));

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

<div class="grid h-full w-full select-none grid-cols-3 grid-rows-3 gap-10">
	<div class="flex h-full w-full items-center justify-center" style="grid-column: 1; grid-row: 1;">
		{#await getName()}
			<label class="label w-1/2">
				<label for="name" class="label-text">Name</label>
				<input id="name" readonly class="input preset-outlined-primary-500 dark:preset-tonal" value={'Loading...'} />
			</label>
		{:then name}
			<label class="label w-1/2">
				<label for="name" class="label-text">Name</label>
				<input id="name" readonly class="input preset-outlined-primary-500 dark:preset-tonal" value={name} />
			</label>
		{:catch error}
			<label class="label w-1/2">
				<label for="name" class="label-text">Name</label>
				<input id="name" readonly class="input preset-filled-error-500" value={error.message} />
			</label>
		{/await}
	</div>
	<div class="flex h-full w-full items-center justify-center" style="grid-column: 1; grid-row: 2;">
		{#await getVersion()}
			<label class="label w-1/2">
				<label for="aV" class="label-text">App Version</label>
				<input id="aV" readonly class="input preset-outlined-primary-700 dark:preset-tonal" value={'Loading...'} />
			</label>
		{:then aVersion}
			<label class="label w-1/2">
				<label for="aV" class="label-text">App Version</label>
				<input id="aV" readonly class="input preset-outlined-primary-700 dark:preset-tonal" value={aVersion} />
			</label>
		{:catch error}
			<label class="label w-1/2">
				<label for="aV" class="label-text">App Version</label>
				<input id="aV" readonly class="input preset-filled-error-500" value={error.message} />
			</label>
		{/await}
	</div>
	<div class="flex h-full w-full items-center justify-center" style="grid-column: 1; grid-row: 3;">
		{#await getTauriVersion()}
			<label class="label w-1/2">
				<label for="tV" class="label-text">Tauri Version</label>
				<input id="tV" readonly class="input preset-outlined-primary-900 dark:preset-tonal" value={'Loading...'} />
			</label>
		{:then tVersion}
			<label class="label w-1/2">
				<label for="tV" class="label-text">Tauri Version</label>
				<input id="tV" readonly class="input preset-outlined-primary-900 dark:preset-tonal" value={tVersion} />
			</label>
		{:catch error}
			<label class="label w-1/2">
				<label for="tV" class="label-text">Tauri Version</label>
				<input id="tV" readonly class="input preset-filled-error-500" value={error.message} />
			</label>
		{/await}
	</div>
	<div class="flex h-full w-full items-center justify-center" style="grid-column: 2; grid-row: 1;">
		<label class="label">
			<label for="theme" class="label-text">Theme</label>
			<select
				id="theme"
				class="select w-1/2 preset-outlined-primary-500 dark:preset-tonal"
				bind:value={$appData.theme}
			>
				{#each themeEntries as [k, v]}
					<option class="capitalize" value={v}>{convertToTitleCase(k)}</option>
				{/each}
			</select>
		</label>
	</div>
	<div class="flex h-full w-full items-center justify-center" style="grid-column: 2; grid-row: 2;">
		<label class="label">
			<label for="feature" class="label-text">Experimental Features</label>
			<input
				id="feature"
				class="select w-1/2 preset-outlined-primary-700 dark:preset-tonal"
				bind:value={featureInput}
			/>
		</label>
	</div>
	{#if ffIsEnabled($appData, FeatureFlag.CHANGE_LANGUAGE)}
		<div
			class="flex h-full w-full items-center justify-center"
			style="grid-column: 2; grid-row: 3;"
		>
			<label class="label">
				<label for="lang" class="label-text"
					>Language <span class="label-experimental">Experimental</span></label
				>
				<select
					id="lang"
					class="select w-1/2 preset-outlined-primary-900 dark:preset-tonal"
					bind:value={$appData.language}
				>
					{#each langEntries as [k, v]}
						<option class="capitalize" value={v}>{convertToTitleCase(k)}</option>
					{/each}
				</select>
			</label>
		</div>
	{/if}
	<!-- TODO: Add autofill for converter (create folder, target path, conversion type (Modal?) -->
</div>
