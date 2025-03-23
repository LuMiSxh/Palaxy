<script lang="ts">
	import { t } from 'svelte-i18n-lingui';

	let images = ['/holo/watame.png', '/holo/suisei.png'];

	let currentImage = $derived(images[Math.floor(Math.random() * images.length)]);
</script>

<div class="grid h-full w-full grid-cols-2 grid-rows-1 gap-2">
	<div class="h-full w-full p-4">
		<h1
			class="text-primary from-primary via-secondary to-secondary mb-2 bg-gradient-to-r via-60% bg-clip-text text-4xl dark:text-transparent"
		>
			{$t`Welcome to Palaxy!`}
		</h1>
		<p class="pb-2 text-lg">
			{$t`Manga conversion and search at your fingertips.`}
		</p>
	</div>
	<div class="flex items-end justify-center">
		<img src={currentImage} class="max-h-full max-w-full object-contain" alt="Character" />
	</div>
</div>
