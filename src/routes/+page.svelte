<script lang="ts">
	import { t } from 'svelte-i18n-lingui';
	import { onMount } from 'svelte';
	import { keyHint } from '$states/keyhint.svelte';
	import KeyComboDisplay from '$components/KeyComboDisplay.svelte';

	let images = ['/holo/watame.png', '/holo/suisei.png', '/holo/towa.png', '/holo/laplus.png'];

	let currentImage = $derived(images[Math.floor(Math.random() * images.length)]);

	// Reset all key hints
	onMount(() => {
		keyHint.clear();
		keyHint.addKey('space', $t`ActionHub`);
	});
</script>

<div class="grid h-full w-full grid-cols-2 grid-rows-1 gap-2">
	<div class="h-full w-full p-4">
		<h1
			class="from-primary via-secondary to-secondary mb-2 bg-gradient-to-r via-60% bg-clip-text text-4xl text-transparent"
		>
			{$t`Welcome to Palaxy!`}
		</h1>
		<p class="pb-2 text-lg">
			{$t`Manga conversion and search at your fingertips.`}
		</p>
		<p>
			{$t`To ge started press`}
			<KeyComboDisplay keyCombination="space" />
			<br />
			{$t`This will open the ActionHub from where you can access all the features of Palaxy.`}
		</p>
	</div>
	<div class="flex items-end justify-end">
		<img
			src={currentImage}
			class="fade-left-image max-h-full max-w-full object-contain"
			alt="Character"
		/>
	</div>
</div>

<style>
	.fade-left-image {
		mask-image: linear-gradient(to right, transparent 0%, black 20%);
	}
</style>
