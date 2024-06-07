<script lang="ts">
	import { converterTab, tabDisableBack, tabDisableNext } from '$lib/stores';

	export let title: string | undefined = undefined;
	export let instruction: string | undefined = undefined;

	export let index: number = 0;
	export let length: number = 0;
</script>

<div class="card min-h-[70vh] flex items-start px-3 mx-4 py-0 justify-center flex-col">
	<!-- Top -->
	<div class="card-header pt-0">
		{#if title}
			<h1
				class="text-3xl font-bold bg-gradient-to-bl from-secondary-500 to-purple-600 bg-clip-text text-transparent box-decoration-clone"
			>
				{title}
			</h1>
		{/if}
		{#if instruction}
			<p class="text-lg">{instruction}</p>
		{:else}
			<br />
		{/if}
	</div>
	<!-- Tab Element -->
	<div class="w-full px-2 min-h-[45vh] flex items-center justify-center">
		<slot />
	</div>
	<!-- Bottom -->
	<div class="card-footer pb-0 w-full flex justify-between">
		{#if index > 0}
			<button
				on:click={() => converterTab.update((tab) => tab - 1)}
				disabled={$tabDisableBack}
				class="btn variant-ghost-secondary btn-lg"
			>
				Go Back
			</button>
		{:else}
			<div class="w-0.5"></div>
		{/if}
		{#if index < length - 1}
			<button
				disabled={$tabDisableNext}
				on:click={() => converterTab.update((tab) => tab + 1)}
				class="btn variant-ghost-primary btn-lg"
			>
				Next Step
			</button>
		{:else}
			<div class="w-0.5"></div>
		{/if}
	</div>
</div>
