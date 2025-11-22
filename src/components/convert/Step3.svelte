<script lang="ts">
	import { appData } from '$stores/appdata';
	import convState, { stepState } from '$states/converter.svelte';
	import { type BundleFlag, commands, type Direction, type FileFormat } from '$types';
	import { t } from 'svelte-i18n-lingui';
	import { IconFolder } from '@tabler/icons-svelte';
	import { open } from '@tauri-apps/plugin-dialog';
	import { onDestroy, onMount } from 'svelte';
	import { truncatePath, wrapper } from '$lib/utils';
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
	let convertToWebp = $state(true);
	let bundleFlag: BundleFlag = $state(
		convState.bundle ? convState.bundle : convState.bundleRecommendation
	);

	async function select() {
		targetLocation =
			(await open({
				directory: true,
				multiple: false
			})) ?? '';
	}

	onMount(() => {
		return keyHint.smartAdd([['tab', $t`Navigate fields`]]);
	});

	onDestroy(async () => {
		// Set convState values
		convState.name = name;
		convState.bundle = bundleFlag;
		convState.target = targetLocation;

		// Set Tauri AppState
		await wrapper(commands.convStateSet({ BundleFlag: bundleFlag }));
		await wrapper(commands.convStateSet({ Name: name ?? '' }));
		await wrapper(commands.convStateSet({ Direction: readingDirection }));
		await wrapper(commands.convStateSet({ Format: fileFormat }));
		await wrapper(commands.convStateSet({ CreateDirectory: createFolder }));
		await wrapper(commands.convStateSet({ ConvertToWebp: convertToWebp }));
		await wrapper(commands.convStateSet({ Target: targetLocation }));
	});

	$effect(() => {
		stepState.disableNext =
			targetLocation === '' ||
			(bundleFlag === 'IMAGE' && convState.bundleRecommendation === 'MANUAL') ||
			name === '';
	});
</script>

<div class="flex h-full w-full flex-col overflow-y-auto p-4" style="max-height: calc(100vh - 8rem)">
	<div class="grid h-full w-full grid-cols-2 gap-4">
		<!-- Basic Settings Card -->
		<div class="card">
			<div class="card-header">
				<h3 class="font-semibold">{$t`Project Settings`}</h3>
			</div>
			<div class="card-body space-y-4">
				<!-- Project Name -->
				<div class="">
					<label for="project-name" class="mb-2 block font-medium">{$t`Project Name`}</label>
					<input
						id="project-name"
						type="text"
						class="input input-primary"
						bind:value={name}
						placeholder={$t`Enter project name`}
					/>
				</div>

				<!-- Bundle Type -->
				<div class="">
					<label for="bundle-type" class="mb-2 block font-medium">{$t`Bundle Type`}</label>
					<select
						id="bundle-type"
						class="select select-primary"
						use:handleKeyHint={{
							keys: [
								['arrowup', $t`Select up`],
								['arrowdown', $t`Select down`],
								['enter', $t`Select`]
							],
							reset: true
						}}
						bind:value={bundleFlag}
					>
						<option selected={bundleFlag === 'MANUAL'} value="MANUAL">{$t`Manual`}</option>
						<option selected={bundleFlag === 'IMAGE'} value="IMAGE">{$t`Image`}</option>
						{#if convState.bundleRecommendation !== 'MANUAL'}
							<option selected={bundleFlag === 'NAME'} value="NAME">{$t`Automatic`}</option>
						{/if}
					</select>
				</div>
			</div>
		</div>

		<!-- Output Settings Card -->
		<div class="card">
			<div class="card-header">
				<h3 class="font-semibold">{$t`Output Settings`}</h3>
			</div>
			<div class="card-body space-y-4">
				<!-- File Type -->
				<div class="">
					<label for="file-type" class="mb-2 block font-medium">{$t`File Type`}</label>
					<select
						id="file-type"
						class="select select-primary"
						use:handleKeyHint={{
							keys: [
								['arrowup', $t`Select up`],
								['arrowdown', $t`Select down`],
								['enter', $t`Select`]
							],
							reset: true
						}}
						bind:value={fileFormat}
					>
						<option selected={fileFormat === 'EPUB'} value="EPUB">{$t`EPUB`}</option>
						<option selected={fileFormat === 'CBZ'} value="CBZ">{$t`CBZ`}</option>
					</select>
				</div>

				<!-- Reading Direction -->
				<div class="">
					<label for="reading-direction" class="mb-2 block font-medium">
						<span class="flex items-center">
							<span>{$t`Reading Direction`}</span>
							{#if fileFormat !== 'EPUB'}
								<span class="badge badge-secondary ml-2">{$t`EPUB only`}</span>
							{/if}
						</span>
					</label>
					<select
						id="reading-direction"
						class="select select-primary"
						disabled={fileFormat !== 'EPUB'}
						use:handleKeyHint={{
							keys: [
								['arrowup', $t`Select up`],
								['arrowdown', $t`Select down`],
								['enter', $t`Select`]
							],
							reset: true
						}}
						bind:value={readingDirection}
					>
						<option selected={readingDirection === 'Left to Right'} value="Left to Right">
							{$t`Left to Right`}
						</option>
						<option selected={readingDirection === 'Right to Left'} value="Right to Left">
							{$t`Right to Left`}
						</option>
					</select>
				</div>

				<!-- Target Location -->
				<div class="">
					<label for="target-location" class="mb-2 block font-medium">{$t`Target Location`}</label>
					<div class="flex gap-3">
						<button
							id="target-location"
							class="btn btn-primary flex-1 justify-between overflow-hidden"
							use:handleKeyHint={{ keys: [['enter', $t`Select location`]] }}
							onclick={select}
						>
							<span class="flex items-center">
								<IconFolder class="mr-2" size={20} />
								{#if !targetLocation}
									<span>{$t`Select output folder`}</span>
								{:else}
									<span class="overflow-hidden text-ellipsis"
										>{truncatePath(targetLocation).split('/').pop()}</span
									>
								{/if}
							</span>
						</button>
					</div>
					{#if targetLocation}
						<p
							class="text-content-secondary dark:text-content-dark-secondary mt-1 truncate text-xs"
							title={targetLocation}
						>
							{truncatePath(targetLocation, 60)}
						</p>
					{/if}
				</div>

				<!-- Create Folder -->
				<div class="">
					<label for="create-folder" class="mb-2 block font-medium">{$t`Create New Folder`}</label>
					<div class="flex items-center gap-4">
						<label class="toggle toggle-lg">
							<input
								id="create-folder"
								type="checkbox"
								class="toggle-input"
								use:handleKeyHint={{ keys: [['enter', $t`Toggle`]] }}
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
						<span class="text-sm">
							{createFolder
								? $t`Will create a new folder for output files`
								: $t`Will save directly in target location`}
						</span>
					</div>
				</div>

				<!-- Convert to WebP -->
				<div class="">
					<label for="convert-webp" class="mb-2 block font-medium">{$t`Convert to WebP`}</label>
					<div class="flex items-center gap-4">
						<label class="toggle toggle-lg">
							<input
								id="convert-webp"
								type="checkbox"
								class="toggle-input"
								use:handleKeyHint={{ keys: [['enter', $t`Toggle`]] }}
								bind:checked={convertToWebp}
								onkeydown={(evt) => {
									if (evt.key === 'Enter') {
										convertToWebp = !convertToWebp;
									}
								}}
							/>
							<span class="toggle-track">
								<span class="toggle-thumb"></span>
							</span>
						</label>
						<span class="text-sm">
							{convertToWebp
								? $t`Images will be converted to WebP format`
								: $t`Images will keep their original format`}
						</span>
					</div>
				</div>
			</div>
		</div>
	</div>
</div>
