<script lang="ts">
	import "../app.postcss"
	import {
		AppBar,
		AppShell,
		getToastStore,
		initializeStores,
		setModeCurrent,
		storePopup,
		TabAnchor,
		TabGroup,
		Toast,
	} from "@skeletonlabs/skeleton"

	import { page } from "$app/stores"
	// Floating UI for Popups
	import { arrow, autoUpdate, computePosition, flip, offset, shift } from "@floating-ui/dom"
	// ---
	import { appDataDefault, appDataKey } from "$lib/constants"
	import { appData, toast } from "$lib/stores"
	import { Theme } from "$lib/types"
	import { generateToast, setTheme } from "$lib/functions"
	import { IconSettingsFilled, IconTransformFilled } from "@tabler/icons-svelte"

	// Popup
	storePopup.set({ computePosition, autoUpdate, flip, shift, offset, arrow })

	// Toast
	initializeStores()
	const toastStore = getToastStore()

	toast.subscribe(t => {
		if (!t) return

		const toastSettings = generateToast(t)
		toastStore.trigger(toastSettings)
		toast.set(undefined)
	})

	if (window && document) {
		// Load AppData
		const appDataValue = localStorage.getItem(appDataKey)
		if (appDataValue) {
			appData.set(JSON.parse(appDataValue))
		} else {
			appData.set(appDataDefault)
		}

		// Set dark / light mode
	}

	// Theme
	setTheme($appData.theme)

	if (window) {
		window.matchMedia("(prefers-color-scheme: dark)").addEventListener("change", event => {
			const newColorScheme = event.matches

			if ($appData.theme === Theme.SYSTEM) {
				setModeCurrent(!newColorScheme)
			}
		})
	}

	const paths = [
		{
			path: "/",
			icon: IconTransformFilled,
		},
		{
			path: "/settings",
			icon: IconSettingsFilled,
		},
	]
</script>

<Toast />
<AppShell class="!h-screen !max-h-screen">
	<AppBar slot="header" class="bg-surface-50 py-2 shadow-2xl dark:bg-surface-500">
		<a href="/" slot="lead">
			<img src="/favicon.png" class="aspect-square h-[3.5rem] select-none" alt="Logo" />
		</a>
		<h1
			class="select-none bg-gradient-to-br from-primary-500 to-secondary-500 box-decoration-clone bg-clip-text pb-0.5 text-5xl text-transparent dark:from-tertiary-500 dark:via-primary-500"
		>
			Palaxy
		</h1>
		<TabGroup
			slot="trail"
			justify="justify-center"
			active="variant-ghost-primary"
			hover="hover:variant-soft-primary"
			flex="flex-1 lg:flex-none"
			rounded="rounded-md"
			border=""
			class="w-full"
		>
			{#each paths as path}
				<TabAnchor
					class="group mx-1 flex items-center justify-center text-center"
					href={path.path}
					selected={$page.url.pathname === path.path}
				>
					<svelte:component
						this={path.icon}
						size="28"
						class="group-hover:text-black group-hover:dark:text-white"
					/>
				</TabAnchor>
			{/each}
		</TabGroup>
	</AppBar>
	<slot />
</AppShell>
