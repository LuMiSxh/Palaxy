<script lang="ts">
	import type { Writable } from 'svelte/store';
	import BookUpload from '@tabler/icons-svelte/IconBookUpload.svelte';
	import { open } from '@tauri-apps/api/dialog';
	import { converterTab } from '$lib/stores';

	export let sourceManga: Writable<string | null>;

	async function select() {
		$sourceManga = await open({
			directory: true,
			multiple: false
		}) as string | null;
	}
</script>

<div class="w-full min-h-full flex items-center justify-center flex-col">
	<h4 class="h4">Please select a directory to use as source material</h4>
	<button
		class="btn variant-ghost-secondary m-8"
		on:click={select}
	>
		<BookUpload class="w-12 h-12" />
	</button>
	<h4 class="h4 text-center">
		Selected Directory: <br />
		{#if $sourceManga === null}
			<code class="code">No directory selected</code>
		{:else}
			<code class="code">{$sourceManga}</code>
		{/if}
	</h4>
	<div class="w-full flex justify-evenly pt-4">
		<button
			on:click={() => converterTab.update(tab => tab-1)}
			disabled={true}
			class="btn variant-ghost-secondary btn-lg"
		>
			Go Back
		</button>
		<button
			disabled={$sourceManga === null}
			on:click={() => converterTab.update(tab => tab+1)}
			class="btn variant-ghost-primary btn-lg"
		>
			Next Step
		</button>
	</div>
</div>