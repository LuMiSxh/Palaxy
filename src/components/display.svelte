<script lang="ts">
	import type { PopupSettings } from "@skeletonlabs/skeleton"
	import { popup } from "@skeletonlabs/skeleton"
	import { appData } from "$lib/stores"
	import { generateRandomString } from "$lib/functions"
	import { IconHelpSquareRounded, IconInfoSquareRounded } from "@tabler/icons-svelte"

	const uuid = generateRandomString()

	export let variant: "info" | "help" = "info"
	export let trigger: "hover" | "click" | "focus-blur" | "focus-click" = "hover"
	export let placement: "top" | "bottom" = "top"
	export let size: number = 18
	export let high: boolean = false
	export let color: string = "text-surface-500-500-token"

	const appDataMap = {
		info: $appData.popups.info,
		help: $appData.popups.help,
	}

	export let title: string | undefined = undefined

	const helpPopup: PopupSettings = {
		event: trigger,
		target: uuid,
		placement: placement,
	}
</script>

{#if appDataMap[variant]}
	<!-- Anchor for popup -->
	<div
		class="h-min w-min cursor-help select-none p-1 {high ? 'relative -top-1' : ''}"
		use:popup={helpPopup}
	>
		{#if variant === "info"}
			<IconInfoSquareRounded {size} class={color} />
		{/if}
		{#if variant === "help"}
			<IconHelpSquareRounded {size} class={color} />
		{/if}
	</div>
	<!-- Popup -->
	<div class="card variant-filled-surface z-50 shadow-xl" data-popup={uuid}>
		{#if title}
			<header class="card-header">
				<h6 class="h6 font-bold">
					{title}
				</h6>
			</header>
		{/if}
		<section class="max-w-md animate-none px-4 py-2 text-sm font-normal">
			<slot />
		</section>
	</div>
{/if}
