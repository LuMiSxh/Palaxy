<script lang="ts">
	import { appData } from '$stores/appdata';
	import { t } from 'svelte-i18n-lingui';
	import {
		IconFileCode2,
		IconFileDownload,
		IconFolder,
		IconSettings,
		IconToggleRight
	} from '@tabler/icons-svelte';
	import { open } from '@tauri-apps/plugin-dialog';
	import dialogManager from '$states/dialog.svelte';
	import { handleKeyHint } from '$states/keyhint.svelte';
	import { addToast } from '$states/toast.svelte';
	import { truncatePath } from '$lib/utils';

	let { id = $bindable(null) } = $props();

	let autoPop = $state($appData.autoPop.enabled);
	let convType = $state($appData.autoPop.converter.conversionType);
	let convLocation = $state($appData.autoPop.converter.targetLocation);
	let convNewFolder = $state($appData.autoPop.converter.createNewFolder);

	// set States
	function save() {
		$appData.autoPop.enabled = autoPop;
		$appData.autoPop.converter.conversionType = convType;
		$appData.autoPop.converter.targetLocation = convLocation === '' ? null : convLocation;
		$appData.autoPop.converter.createNewFolder = convNewFolder;
		addToast($t`Auto-Populate Settings Saved`, 'info');
		dialogManager.closeDialog(id);
	}

	async function selectLocation() {
		const location = await open({
			directory: true,
			multiple: false
		});
		if (location) {
			convLocation = location;
		}
	}
</script>

<div class="p-2">
	<div class="grid gap-4">
		<!-- Enable Auto-Population Card -->
		<div class="flex items-center">
			<IconToggleRight class="text-primary mr-2" size={20} />
			<h3 class="font-semibold">{$t`Enable Auto-Population`}</h3>
		</div>
		<label class="toggle toggle-lg">
			<input
				type="checkbox"
				class="toggle-input"
				onkeydown={(evt) => {
					if (evt.key === 'Enter') {
						evt.preventDefault();
						evt.stopPropagation();
						autoPop = !autoPop;
					}
				}}
				bind:checked={autoPop}
			/>
			<span class="toggle-track">
				<span class="toggle-thumb"></span>
			</span>
		</label>

		<!-- File Type Card -->
		<div class="flex items-center">
			<IconFileCode2 class="text-primary mr-2" size={20} />
			<h3 class="font-semibold">{$t`File Type`}</h3>
		</div>
		<select
			class="select select-primary w-full"
			use:handleKeyHint={{
				keys: [
					['arrowup', $t`Select up`],
					['arrowdown', $t`Select down`],
					['enter', $t`Select`]
				]
			}}
			bind:value={convType}
		>
			<option value="PDF">{$t`PDF`}</option>
			<option value="EPUB">{$t`EPUB`}</option>
			<option value="CBZ">{$t`CBZ`}</option>
			<option value={null}>{$t`None`}</option>
		</select>

		<!-- Target Location Card -->
		<div class="flex items-center">
			<IconFolder class="text-primary mr-2" size={20} />
			<h3 class="font-semibold">{$t`Target Location`}</h3>
		</div>
		<button
			class="btn btn-primary flex w-full items-center justify-between p-2 select-none"
			onclick={selectLocation}
			onkeydown={(evt) => {
				if (evt.key === 'Enter') {
					evt.preventDefault();
					evt.stopPropagation();
					selectLocation();
				}
			}}
			use:handleKeyHint={{ keys: [['enter', $t`Invoke`]] }}
		>
			<span class="flex items-center">
				<IconFileDownload class="mr-2" size={20} />
				{#if !convLocation}
					<span>{$t`Click to select a folder`}</span>
				{:else}
					<span class="overflow-hidden text-ellipsis">{convLocation.split('/').pop()}</span>
				{/if}
			</span>
		</button>
		{#if convLocation}
			<p
				class="text-content-secondary dark:text-content-dark-secondary mt-2 truncate text-xs"
				title={convLocation}
			>
				{truncatePath(convLocation, 60)}
			</p>
		{/if}

		<!-- Create New Folder Card -->
		<div class="flex items-center">
			<IconSettings class="text-primary mr-2" size={20} />
			<h3 class="font-semibold">{$t`Create New Folder`}</h3>
		</div>
		<div class="flex items-center gap-4">
			<label class="toggle toggle-lg">
				<input
					type="checkbox"
					class="toggle-input"
					onkeydown={(evt) => {
						if (evt.key === 'Enter') {
							evt.preventDefault();
							evt.stopPropagation();
							convNewFolder = !convNewFolder;
						}
					}}
					use:handleKeyHint={{ keys: [['enter', $t`Invoke`]] }}
					bind:checked={convNewFolder}
				/>
				<span class="toggle-track">
					<span class="toggle-thumb"></span>
				</span>
			</label>
			<span class="text-sm">
				{convNewFolder
					? $t`Will create a new folder for output files`
					: $t`Will save directly in target location`}
			</span>
		</div>
	</div>

	<div class="mt-4 flex justify-end gap-2">
		<button
			class="btn btn-soft"
			use:handleKeyHint={{ keys: [['enter', $t`Invoke`]] }}
			onclick={() => dialogManager.closeDialog(id)}
			onkeydown={(evt) => {
				if (evt.key === 'Enter') {
					evt.preventDefault();
					evt.stopPropagation();
					dialogManager.closeDialog(id);
				}
			}}
		>
			{$t`Cancel`}
		</button>
		<button
			class="btn btn-primary"
			use:handleKeyHint={{ keys: [['enter', $t`Invoke`]] }}
			onclick={save}
			onkeydown={(evt) => {
				if (evt.key === 'Enter') {
					evt.preventDefault();
					evt.stopPropagation();
					save();
				}
			}}
		>
			{$t`Save`}
		</button>
	</div>
</div>
