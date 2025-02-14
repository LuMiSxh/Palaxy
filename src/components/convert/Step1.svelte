<script lang="ts">
	import { onMount } from 'svelte';
	import { commands } from '$types';
	import { t } from 'svelte-i18n-lingui';
	import { open } from '@tauri-apps/plugin-dialog';
	import converter, { stepState } from '$states/converter.svelte';
	import { wrapper } from '$lib/utils';
	import { getCurrentWebview } from '@tauri-apps/api/webview';
	import { addToast } from '$stores/toast';
	import { IconFileDownload } from '@tabler/icons-svelte';

	onMount(async () => {
		// Reset
		await wrapper(commands.convReset());
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
				await wrapper(commands.convSetSource(event.payload.paths[0]));
			}
		});
	});

	async function select() {
		converter.source = await open({
			directory: true,
			multiple: false
		});

		if (converter.source !== null) {
			await wrapper(commands.convSetSource(converter.source));
		}
	}

	$effect(() => {
		stepState.disableNext = converter.source === null;
	});
</script>

<button
	id="dropzone"
	class="btn btn-soft flex min-h-1/2 min-w-1/2 flex-col items-center justify-center"
	onclick={select}
>
	<span class="flex items-center justify-center">
		<IconFileDownload class="text-primary" />
		<span>{$t`Click to select a folder or drag it in`}</span>
	</span>
	<span class="divider"></span>
	<span class="flex flex-col items-center justify-center">
		<span class="font-bold">{$t`Selected path`}:</span>
		{#if converter.source}
			<code class="text-primary">{converter.source}</code>
		{:else}
			<span class="text-error">{$t`None selected`}</span>
		{/if}
	</span>
</button>
