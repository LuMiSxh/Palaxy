<script lang="ts">
	import { appData } from '$stores/appdata';
	import convState, { stepState } from '$states/converter.svelte';
	import { type BundleFlag, commands, type Direction, type FileFormat } from '$types';
	import { t } from 'svelte-i18n-lingui';
	import { IconFileDownload } from '@tabler/icons-svelte';
	import { open } from '@tauri-apps/plugin-dialog';
	import { onDestroy, onMount } from 'svelte';
	import { wrapper } from '$lib/utils';
	import { handleKeyHint, keyHint } from '$states/keyhint.svelte';

	let name = $state(
		convState.name === ''
			? convState.source
				? (convState.source.split('/').pop() ?? convState.name)
				: convState.name
			: convState.name
	);
	let targetLocation = $state(
		$appData.autoPop.enabled
			? ($appData.autoPop.converter.targetLocation ?? convState.target ?? '')
			: (convState.target ?? '')
	);
	let createFolder = $state(
		$appData.autoPop.enabled ? ($appData.autoPop.converter.createNewFolder ?? true) : true
	);
	let fileFormat: FileFormat = $state(
		$appData.autoPop.enabled ? ($appData.autoPop.converter.conversionType ?? 'CBZ') : 'CBZ'
	);
	let readingDirection: Direction = $state('Left to Right');
	let bundleFlag: BundleFlag = $state(convState.bundleRecommendation);

	async function select() {
		targetLocation =
			(await open({
				directory: true,
				multiple: false
			})) ?? '';
	}

	onMount(() => {
		return keyHint.smartAdd([
			['tab', $t`Navigate between fields`]
		]);
	});

	onDestroy(async () => {
		// Set convState values
		convState.name = name;
		convState.bundle = bundleFlag;
		convState.target = targetLocation;

		// Set Tauri AppState
		await wrapper(commands.convStateSet({ Name: name ?? '' }));
		await wrapper(commands.convStateSet({ BundleFlag: bundleFlag }));
		await wrapper(commands.convStateSet({ Direction: readingDirection }));
		await wrapper(commands.convStateSet({ Format: fileFormat }));
		await wrapper(commands.convStateSet({ CreateDirectory: createFolder }));
	});

	$effect(() => {
		stepState.disableNext =
			targetLocation === '' ||
			(bundleFlag === 'IMAGE' && convState.bundleRecommendation === 'MANUAL') ||
			name === '';
		// stepState.indexIncrement = bundleType === "MANUAL"? 1 : 2;
	});
</script>

{#snippet option(type: any, value: any, text: string)}
	<option selected={type === value} {value}>{text}</option>
{/snippet}

<div class="grid h-full w-full grid-cols-3 gap-8">
	<div class="h-full w-full flex items-center justify-center ">
		<fieldset class="fieldset w-full m-4">
			<legend class="fieldset-legend">{$t`File Name(s)`}</legend>
			<input type="text" class="input input-primary" bind:value={name} />
		</fieldset>
	</div>
	<div class="h-full w-full flex items-center justify-center">
		<fieldset class="fieldset w-full">
			<legend class="fieldset-legend">{$t`Bundle Type`}</legend>
			<select class="select select-primary"
							use:handleKeyHint={{keys: [['arrowup', $t`Select up`], ['arrowdown', $t`Select down`], ['enter', $t`Select`]], reset: true}}
							bind:value={bundleFlag}>
				{@render option(bundleFlag, 'MANUAL', $t`Manual`)}
				{@render option(bundleFlag, 'IMAGE', $t`Image`)}
				{#if convState.bundleRecommendation !== 'MANUAL'}
					{@render option(bundleFlag, 'NAME', $t`Automatic`)}
				{/if}
			</select>
		</fieldset>
	</div>
	<div class="h-full w-full flex items-center justify-center">
		<fieldset class="fieldset w-full m-4">
			<legend class="fieldset-legend">{$t`File Type`}</legend>
			<select class="select select-secondary"
							use:handleKeyHint={{keys: [['arrowup', $t`Select up`], ['arrowdown', $t`Select down`], ['enter', $t`Select`]], reset: true}}
							bind:value={fileFormat}>
				{@render option(fileFormat, 'PDF', $t`PDF`)}
				{@render option(fileFormat, 'EPUB', $t`EPUB`)}
				{@render option(fileFormat, 'CBZ', $t`CBZ`)}
			</select>
		</fieldset>
	</div>
	<div class="h-full w-full flex items-center justify-center">
		<fieldset class="fieldset w-full m-4">
			<legend class="fieldset-legend">{$t`Reading Direction`}</legend>
			<select
				class="select select-secondary"
				disabled={fileFormat !== 'EPUB'}
				use:handleKeyHint={{keys: [['arrowup', $t`Select up`], ['arrowdown', $t`Select down`], ['enter', $t`Select`]], reset: true}}
				bind:value={readingDirection}
			>
				{@render option(readingDirection, 'Left to Right', $t`Left to Right`)}
				{@render option(readingDirection, 'Right to Left', $t`Right to Left`)}
			</select>
		</fieldset>
	</div>
	<div class="h-full w-full flex items-center justify-center">
		<fieldset class="fieldset w-full m-4">
			<legend class="fieldset-legend">{$t`Target Location`}</legend>
			<button
				id="dropzone"
				class="btn btn-primary flex items-center justify-center p-2 select-none"
				use:handleKeyHint={{keys: [["enter", $t`Invoke`]]}}
				onclick={select}
			>
				{#if !targetLocation}
					<IconFileDownload class="text-primary" />
					<span>{$t`Click to select or drag a folder in`}</span>
				{:else if targetLocation}
					<code class="text-white">{targetLocation.split('/').pop()}</code>
				{:else}
					<span class="text-error">{$t`None selected`}</span>
				{/if}
			</button>
		</fieldset>
	</div>
	<div class="h-full w-full flex items-center justify-center">
		<fieldset class="fieldset w-full m-4">
			<legend class="fieldset-legend">{$t`Create folder`}</legend>
			<div class="flex items-center justify-center">
				<label class="toggle toggle-xl">
					<input
						type="checkbox"
						class="toggle-input"
						use:handleKeyHint={{keys: [["enter", $t`Invoke`]]}}
						bind:checked={createFolder}
						onkeydown={(evt) => {
						if (evt.key === 'Enter') {
							createFolder = !createFolder;
						}
					}}
					/>
					<span class="toggle-track">
					<span class="toggle-thumb"></span>
				</span>
				</label>
			</div>
		</fieldset>
	</div>
</div>
