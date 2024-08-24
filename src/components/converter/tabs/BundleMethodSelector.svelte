<script lang="ts">
	import { BundleFlag } from "$components/converter/types"
	import { ListBox, ListBoxItem } from "@skeletonlabs/skeleton"
	import { IconPhotoScan, IconPrompt, IconTextScan2 } from "@tabler/icons-svelte"
	import { onDestroy } from "svelte"
	import { bridge } from "$lib/functions"
	import {
		bundle,
		bundleRecommendation,
		disableBack,
		disableNext,
	} from "$components/converter/stores"

	// Initial state
	$bundle =
		$bundle === null
			? $bundleRecommendation === BundleFlag.NAME
				? BundleFlag.NAME
				: BundleFlag.MANUAL
			: $bundle
	$disableBack = false
	$disableNext = true

	const unsubscribe = bundle.subscribe(value => {
		$disableNext = value === null
	})

	onDestroy(async () => {
		unsubscribe()
		await bridge("set_bundle_flag", { flag: $bundle })
	})
</script>

<div class="flex min-h-full w-full flex-col items-center justify-center">
	<ListBox
		class="variant-glass-primary mt-2 dark:variant-glass-primary"
		active="dark:variant-ghost-primary variant-filled-primary  dark:text-white"
	>
		<ListBoxItem bind:group={$bundle} name="Manual" value={BundleFlag.MANUAL} class="px-7 py-3">
			<svelte:fragment slot="lead">
				<IconPrompt
					class={$bundle === BundleFlag.MANUAL
						? "text-tertiary-500"
						: "text-secondary-500"}
				/>
			</svelte:fragment>
			Choose volume sizes manually
		</ListBoxItem>
		<ListBoxItem bind:group={$bundle} name="Image" value={BundleFlag.IMAGE} class="px-7 py-3">
			<svelte:fragment slot="lead">
				<IconPhotoScan
					class={$bundle === BundleFlag.IMAGE
						? "text-tertiary-500"
						: "text-secondary-500"}
				/>
			</svelte:fragment>
			Calculate volume sizes based on cover images
		</ListBoxItem>
		<ListBoxItem
			bind:group={$bundle}
			name="Naming"
			value={BundleFlag.NAME}
			disabled={BundleFlag.NAME !== $bundleRecommendation}
			class="px-7 py-3"
		>
			<svelte:fragment slot="lead">
				<IconTextScan2
					class={$bundle === BundleFlag.NAME ? "text-tertiary-500" : "text-secondary-500"}
				/>
			</svelte:fragment>
			Calculate volume sizes based on directory naming convention
		</ListBoxItem>
	</ListBox>
</div>
