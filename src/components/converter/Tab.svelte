<script lang="ts">
	import { IconCaretLeftFilled, IconCaretRightFilled } from "@tabler/icons-svelte"
	import { disableBack, disableNext, index } from "$components/converter/stores"

	export let title: string | undefined = undefined
	export let instruction: string | undefined = undefined

	export let idx: number = 0
	export let length: number = 0
</script>

<div
	class="flex h-full max-h-screen w-full flex-grow flex-col bg-surface-50 shadow-inner dark:bg-surface-600"
>
	<!-- Top -->
	<div class="card-header h-[3rem] pt-0">
		{#if title}
			<h1
				class="bg-gradient-to-r from-primary-500 to-tertiary-500 to-30% box-decoration-clone bg-clip-text text-3xl font-bold text-transparent"
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
	<div class="flex flex-grow items-center justify-center text-center">
		<slot />
	</div>
	<!-- Bottom -->
	<div class="flex h-[3rem] w-full justify-between px-3 pb-3">
		{#if idx > 0}
			<button
				on:click={() => index.update(tab => tab - 1)}
				disabled={$disableBack}
				class="variant-ghost-secondary btn btn-lg"
			>
				<IconCaretLeftFilled size="24" />
			</button>
		{:else}
			<div class="w-0.5"></div>
		{/if}
		{#if idx < length - 1}
			<button
				disabled={$disableNext}
				on:click={() => index.update(tab => tab + 1)}
				class="variant-ghost-primary btn btn-lg"
			>
				<IconCaretRightFilled size="24" />
			</button>
		{:else}
			<div class="w-0.5"></div>
		{/if}
	</div>
</div>
