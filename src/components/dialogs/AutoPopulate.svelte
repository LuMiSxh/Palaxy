<script lang="ts">
	import { appData } from '$stores/appdata';
	import { t } from 'svelte-i18n-lingui';
	import { IconFileDownload } from '@tabler/icons-svelte';
	import { open } from '@tauri-apps/plugin-dialog';
	import dialogManager from '$states/dialog.svelte';
	import { handleKeyHint } from '$states/keyhint.svelte';
	import { addToast } from '$states/toast.svelte';

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
		addToast(
			$t`Auto-Populate Settings Saved`,
			'info'
		);
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

<div>
	<div class="grid gap-4">
		<fieldset class="fieldset w-full">
			<legend class="fieldset-legend">{$t`Enable Auto-Population`}</legend>
			<div class="flex items-center justify-center">
				<label class="toggle toggle-xl">
					<input
						type="checkbox"
						class="toggle-input"
						onkeydown={(evt) =>{
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
			</div>
		</fieldset>

		<fieldset class="fieldset w-full">
			<legend class="fieldset-legend">{$t`File Type`}</legend>
			<select class="select select-secondary w-full"
							use:handleKeyHint={{keys: [['arrowup', $t`Select up`], ['arrowdown', $t`Select down`], ['enter', $t`Select`]], reset: true}}
							bind:value={convType}
			>
				<option value="PDF">{$t`PDF`}</option>
				<option value="EPUB">{$t`EPUB`}</option>
				<option value="CBZ">{$t`CBZ`}</option>
				<option value={null}>{$t`None`}</option>
			</select>
		</fieldset>

		<fieldset class="fieldset w-full">
			<legend class="fieldset-legend">{$t`Target Location`}</legend>
			<button
				class="btn btn-primary flex items-center justify-center p-2 select-none w-full"
				onclick={selectLocation}
				onkeydown={(evt) =>{
					if (evt.key === 'Enter') {
						evt.preventDefault();
						evt.stopPropagation();
						selectLocation();
					}
				}}
				use:handleKeyHint={{keys: [["enter", $t`Invoke`]]}}
			>
				{#if !convLocation}
					<IconFileDownload class="text-primary" />
					<span>{$t`Click to select a folder`}</span>
				{:else}
					<code class="text-white">{convLocation.split('/').pop()}</code>
				{/if}
			</button>
		</fieldset>

		<fieldset class="fieldset w-full">
			<legend class="fieldset-legend">{$t`Create New Folder`}</legend>
			<div class="flex items-center justify-center">
				<label class="toggle toggle-xl">
					<input
						type="checkbox"
						class="toggle-input"
						onkeydown={(evt) =>{
							if (evt.key === 'Enter') {
								evt.preventDefault();
								evt.stopPropagation();
								convNewFolder = !convNewFolder;
							}
						}}
						use:handleKeyHint={{keys: [["enter", $t`Invoke`]]}}
						bind:checked={convNewFolder}
					/>
					<span class="toggle-track">
            <span class="toggle-thumb"></span>
          </span>
				</label>
			</div>
		</fieldset>
	</div>

	<div class="flex justify-end gap-2 mt-4">
		<button class="btn btn-neutral"
						use:handleKeyHint={{keys: [["enter", $t`Invoke`]]}}
						onclick={() => dialogManager.closeDialog(id)}
						onkeydown={(evt) =>{
							if (evt.key === 'Enter') {
								evt.preventDefault();
								evt.stopPropagation();
								dialogManager.closeDialog(id);
							}
						}}
		>
			{$t`Cancel`}
		</button>
		<button class="btn btn-primary"
						use:handleKeyHint={{keys: [["enter", $t`Invoke`]]}}
						onclick={save}
						onkeydown={(evt) =>{
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
