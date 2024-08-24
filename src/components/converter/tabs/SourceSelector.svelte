<script lang="ts">
	import { open } from "@tauri-apps/plugin-dialog"
	import { BundleFlag } from "$components/converter/types"
	import { IconFolderFilled } from "@tabler/icons-svelte"
	import { bridge } from "$lib/functions"
	import { onDestroy, onMount } from "svelte"
	import {
		bundle,
		bundleRecommendation,
		chapterSizes,
		disableNext,
		source,
	} from "$components/converter/stores"

	// Reset ALL writable stores and the tauri AppState
	$source = null
	$bundleRecommendation = BundleFlag.MANUAL
	$bundle = null
	$chapterSizes = []

	// Initial state
	$disableNext = true

	onMount(async () => {
		await bridge("reset")
	})

	const unsubscribe = source.subscribe(value => {
		$disableNext = value === null
	})

	onDestroy(() => {
		unsubscribe()
	})

	async function select() {
		$source = (await open({
			directory: true,
			multiple: false,
		})) as string | null

		if ($source !== null) await bridge("set_source", { source: $source })
	}
</script>

<div class="grid w-1/2 grid-cols-1 grid-rows-2 gap-4 p-8">
	<button class="variant-filled-primary btn dark:variant-ghost-primary" on:click={select}>
		<IconFolderFilled size="24" class="text-white" />
	</button>
	<code
		class="code variant-glass-primary flex items-center justify-center rounded-xl p-0.5 text-center text-black dark:variant-glass-primary dark:text-white"
	>
		{#if $source === null}
			No directory selected
		{:else}
			{$source}
		{/if}
	</code>
</div>
