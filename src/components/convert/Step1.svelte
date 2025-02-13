<script lang="ts">
	import { onMount } from 'svelte';
	import { commands } from '$types';
	import { open } from '@tauri-apps/plugin-dialog';
	import converter from '$states/converter.svelte';
	import { wrapper } from '$lib/utils';

	onMount(async () => {
		await wrapper(commands.convReset());
		converter.reset();
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

</script>

<h1>Step 1</h1>
<button onclick={select}>Click me</button>

