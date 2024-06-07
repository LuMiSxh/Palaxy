<script lang="ts">
	import type { Writable } from 'svelte/store';
	import BookUpload from '@tabler/icons-svelte/IconBookUpload.svelte';
	import { open } from '@tauri-apps/api/dialog';
	import { tabDisableBack, tabDisableNext } from '$lib/stores';
	import { BundlerFlag } from '$components/converter/types';

	export let sourceManga: Writable<string | null>;
	export let bundlerRecommendation: Writable<BundlerFlag>;
	export let bundler: Writable<BundlerFlag | null>;
	export let cpv: Writable<Array<number>>;

	// Initial state
	$tabDisableNext = true;
	$tabDisableBack = true;

	async function select() {
		$sourceManga = (await open({
			directory: true,
			multiple: false
		})) as string | null;
	}

	$: if ($sourceManga !== null) {
		$tabDisableNext = false;
	}

	// Reset ALL writable stores
	$sourceManga = null;
	$bundlerRecommendation = BundlerFlag.IMAGE;
	$bundler = null;
	$cpv = [];
</script>

<div class="w-full min-h-full flex items-center justify-center flex-col">
	<button class="btn variant-ghost-secondary mb-4" on:click={select}>
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
</div>
