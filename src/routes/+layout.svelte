<script lang="ts">
	import '../app.css';
	import { page } from '$app/state';
	import { onNavigate } from '$app/navigation';
	import { appData, appDataKey } from '$stores/appdata.js';
	import { defaultAppData, Theme } from '$types/appdata';
	import { setTheme } from '$lib/utils';
	import { browser } from '$app/environment';
	import { fade } from "svelte/transition";

	let { children } = $props();

	const pathMap: Record<string, string> = {
		'/': 'Home',
		'/convert': 'Convert',
		'/search': 'Search',
		'/agents': 'Agents',
		'/settings': 'Settings'
	};

	let currentPath = $derived(pathMap[page.route.id === null ? '/' : page.route.id]);

	onNavigate((navigation) => {
		if (!document.startViewTransition) return;

		return new Promise((resolve) => {
			document.startViewTransition(async () => {
				resolve();
				await navigation.complete;
			});
		});
	});

	if (browser) {
		// Load AppData
		const appDataValue = localStorage.getItem(appDataKey);
		if (appDataValue) {
			appData.set(JSON.parse(appDataValue));
		} else {
			appData.set(defaultAppData);
		}

		window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', (event) => {
			const newColorScheme = event.matches;

			if ($appData.theme === Theme.SYSTEM) {
				setTheme(newColorScheme ? Theme.DARK : Theme.LIGHT);
			}
		});

		// Set dark / light mode
		setTheme($appData.theme);
	}

	// TODO: Redraw the images so they are not cut off
</script>

<div class="flex h-screen max-h-screen w-full flex-col overflow-hidden">
	<div class="navbar bg-base-100 shadow-sm select-none" style="view-transition-name: disabled;">
		<a href="/">
			<img src="/icon.png" alt="Logo" class="!mr-0 aspect-square size-[2.75rem] select-none" />
		</a>
		<div class="flex-1"></div>
		<div class="flex-none relative">
			{#key currentPath}
				<div class="flex h-full items-center justify-center absolute top-0 right-0 whitespace-nowrap" transition:fade={{duration: 120}}>
					{#if currentPath === 'Home'}
						<h6
							class="from-primary to-secondary bg-linear-to-r via-50% bg-clip-text text-3xl font-medium text-transparent drop-shadow-lg dark:drop-shadow-none"
						>
							Palaxy
						</h6>
					{:else}
						<h6 class="text-3xl font-medium">
						<span
							class="from-primary to-secondary bg-linear-to-r via-50% bg-clip-text text-transparent drop-shadow-lg dark:drop-shadow-none"
						>
							Palaxy
						</span>
							| {currentPath}
						</h6>
					{/if}
				</div>
			{/key}
		</div>
	</div>

	<main class="relative h-full w-full grow overflow-auto px-3 py-2">
		<img
			src="/chars/alya.png"
			alt=""
			class="absolute top-0 left-0 -z-50 max-h-full drop-shadow-xs"
		/>
		<img
			src="/chars/masachika.png"
			alt=""
			class="absolute top-0 right-0 -z-50 max-h-full drop-shadow-xs"
		/>
		<div class="h-full">
			{@render children()}
		</div>
	</main>
</div>

<style>
	@keyframes fade-in {
		from {
			opacity: 0;
		}
	}

	@keyframes fade-out {
		to {
			opacity: 0;
		}
	}

	@keyframes slide-from-right {
		from {
			transform: translateX(25vw);
		}
	}

	@keyframes slide-to-left {
		to {
			transform: translateX(-25vw);
		}
	}

	:root::view-transition-old(root) {
		animation:
			90ms cubic-bezier(0.4, 0, 1, 1) both fade-out,
			300ms cubic-bezier(0.4, 0, 0.2, 1) both slide-to-left;
	}

	:root::view-transition-new(root) {
		animation:
			210ms cubic-bezier(0, 0, 0.2, 1) 90ms both fade-in,
			300ms cubic-bezier(0.4, 0, 0.2, 1) both slide-from-right;
	}

  :root::view-transition-group(disabled),
  :root::view-transition-old(disabled),
  :root::view-transition-new(disabled) {
      animation-duration: 0s !important;
  }
</style>
