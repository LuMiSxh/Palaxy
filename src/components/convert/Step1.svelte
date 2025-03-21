<script lang="ts">
	import { onMount } from 'svelte';
	import { commands } from '$types';
	import { t } from 'svelte-i18n-lingui';
	import { open } from '@tauri-apps/plugin-dialog';
	import converter from '$states/converter.svelte';
	import convState, { stepState } from '$states/converter.svelte';
	import { wrapper } from '$lib/utils';
	import { getCurrentWebview } from '@tauri-apps/api/webview';
	import { addToast } from '$states/toast.svelte';
	import { IconFileDownload } from '@tabler/icons-svelte';

	onMount(async () => {
		// Reset
		await wrapper(commands.convStateReset());
		converter.reset();
		stepState.reset();

		const webview = getCurrentWebview();
		await webview.onDragDropEvent(async (event) => {
			if (event.payload.type === 'drop') {
				// get the coordinates of the dropzone
				const dropzone = document.getElementById('dropzone');
				if (dropzone === null) return;
				const rect = dropzone.getBoundingClientRect();

				// check if the drop event happened outside the dropzone
				if (
					event.payload.position.x < rect.left ||
					event.payload.position.x > rect.right ||
					event.payload.position.y < rect.top ||
					event.payload.position.y > rect.bottom
				) {
					return;
				}

				// When there are multiple files toast it
				if (event.payload.paths.length > 1) {
					addToast(
						`Multiple paths dropped, only the first path will be used: <code>${event.payload.paths[0]}</code>`,
						'warning',
						3600
					);
				}

				converter.source = event.payload.paths[0];
				await wrapper(commands.convStateSet({ Source: convState.source ?? '' }));
			}
		});
	});

	async function select() {
		converter.source = await open({
			directory: true,
			multiple: false
		});

		if (converter.source !== null) {
			await wrapper(commands.convStateSet({ Source: convState.source ?? '' }));
		}
	}

	$effect(() => {
		stepState.disableNext = converter.source === null;
	});
</script>

<fieldset class="fieldset">
	<legend class="fieldset-legend">{$t`Source Location`}</legend>
	<button
		id="dropzone"
		class="btn btn-soft flex items-center justify-center p-2 select-none"
		onclick={select}
	>
		{#if !converter.source}
			<IconFileDownload class="text-primary" />
			<span>{$t`Click to select or drag it onto it`}</span>
		{:else if converter.source}
			<code class="text-primary">{converter.source.split('/').pop()}</code>
		{:else}
			<span class="text-error">{$t`None selected`}</span>
		{/if}
	</button>
</fieldset>
