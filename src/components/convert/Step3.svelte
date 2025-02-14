<script lang="ts">
	// Setup a variables to be set
	import { appData } from '$stores/appdata';
	import convState from '$states/converter.svelte';
	import type { BundleFlag, Direction, FileFormat } from '$types';
	import { t } from 'svelte-i18n-lingui';

	let targetLocation = $state($appData.autoPop.enabled ? $appData.autoPop.converter.targetLocation ?? '' : '');
	let createFolder = $state($appData.autoPop.enabled ? $appData.autoPop.converter.createNewFolder ?? true : true);
	let conversionType: FileFormat = $state($appData.autoPop.enabled ? $appData.autoPop.converter.conversionType ?? 'CBZ' : 'CBZ');
	let readingDirection: Direction = $state('Left to Right');
	let bundleType: BundleFlag = $state(convState.bundleRecommendation);
	console.log(convState.bundleRecommendation);

	const classUnchecked = '';
</script>

{#snippet option(type: any, value: any, text: string)}
	<option selected={type === value} value={value}>{text}</option>
{/snippet}

<div>
	<fieldset class="fieldset">
		<legend class="fieldset-legend">{$t`Bundle Type`}</legend>
		<select class="select select-primary" bind:value={bundleType}>
			{@render option(bundleType, "MANUAL", $t`Manual`)}
			{@render option(bundleType, "IMAGE", $t`Image`)}
			{#if convState.bundleRecommendation !== "MANUAL"}
				{@render option(bundleType, "NAME", $t`Automatic`)}
			{/if}
		</select>
	</fieldset>
	<fieldset class="fieldset">
		<legend class="fieldset-legend">{$t`Reading Direction`}</legend>
		<select class="select select-secondary" disabled={conversionType !== "PDF"} bind:value={readingDirection}>
			{@render option(readingDirection, "Left to Right", $t`Left to Right`)}
			{@render option(readingDirection, "Right to Left", $t`Right to Left`)}
		</select>
	</fieldset>
	<fieldset class="fieldset">
		<legend class="fieldset-legend">{$t`File Type`}</legend>
		<select class="select select-secondary" bind:value={conversionType}>
			{@render option(conversionType, "PDF", $t`PDF`)}
			{@render option(conversionType, "EPUB", $t`EPUB`)}
			{@render option(conversionType, "CBZ", $t`CBZ`)}
		</select>
	</fieldset>
	<fieldset class="fieldset">
		<legend class="fieldset-legend">{$t`Create folder`}</legend>
		<label class="toggle toggle-lg select-none" class:border-accent={createFolder} class:before:!text-accent={createFolder}>
			<input type="checkbox" hidden bind:checked={createFolder}>
		</label>
	</fieldset>
</div>
