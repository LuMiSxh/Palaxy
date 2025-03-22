<script lang="ts">
	import { t } from 'svelte-i18n-lingui';

	let images = [
		'/holo/watame.png',
		'/holo/suisei.png'
	];

	let currentImage = $derived(images[Math.floor(Math.random() * images.length)]);
</script>

<div class="w-full h-full grid grid-cols-2 grid-rows-1 gap-2">
	<div class="w-full h-full p-4">
		<h1
			class="dark:text-transparent text-primary bg-clip-text bg-gradient-to-r from-primary via-secondary via-60% to-secondary text-4xl mb-2">
			{$t`Welcome to Palaxy!`}
		</h1>
		<p class="pb-2 text-lg">
			{$t`Manga conversion and search at your fingertips.`}
		</p>
	</div>
	<div class="flex items-end justify-center">
		<img src={currentImage} class="max-w-full max-h-full object-contain" alt="Character" />
	</div>
</div>
