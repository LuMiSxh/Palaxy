<script lang="ts">
	import '../app.css';
	import { page } from '$app/state';
	import { AppBar, ToastProvider } from '@skeletonlabs/skeleton-svelte';
	import { onNavigate } from '$app/navigation';
	import { appData, appDataKey } from '$stores/appdata.js';
	import { defaultAppData, Theme } from '$types/appdata';
	import { setTheme } from '$lib/utils';
	import { browser } from '$app/environment';

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

		window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', event => {
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

<ToastProvider>
	<div class="flex h-screen max-h-screen w-full flex-col overflow-hidden">
		<AppBar base="items-center py-2 px-3">
			{#snippet lead()}
				<a href="/" class="">
					<img src="/icon.png" alt="Logo" class="aspect-square h-[2.75rem] select-none" />
				</a>
			{/snippet}
			{#snippet trail()}
				<div class="flex h-full items-center justify-center">
					{#if currentPath === 'Home'}
						<h6
							class="bg-gradient-to-r from-primary-600 via-primary-700 dark:via-primary-300 via-50% to-primary-900 dark:to-primary-100 bg-clip-text text-3xl font-medium text-transparent"
						>
							Palaxy
						</h6>
					{:else}
						<h6 class="text-3xl font-medium">
							<span
								class="bg-gradient-to-r from-primary-600 via-primary-700 dark:via-primary-300 via-50% to-primary-900 dark:to-primary-100 bg-clip-text text-transparent"
							>
								Palaxy
							</span>
							| {currentPath}
						</h6>
					{/if}
				</div>
			{/snippet}
		</AppBar>
		<main class="h-full w-full flex-grow overflow-auto px-3 py-2 relative">
			<img src="/chars/alya.png" alt="" class="max-h-full -z-50 absolute top-0 left-0 drop-shadow-sm" />
			<img src="/chars/masachika.png" alt="" class="max-h-full -z-50 absolute top-0 right-0 drop-shadow-sm" />
			<div class="h-full">
				{@render children()}
			</div>
		</main>
	</div>
</ToastProvider>

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
            transform: translateX(50vw);
        }
    }

    @keyframes slide-to-left {
        to {
            transform: translateX(-50vw);
        }
    }

    :root::view-transition-old(root) {
        animation: 90ms cubic-bezier(0.4, 0, 1, 1) both fade-out,
        300ms cubic-bezier(0.4, 0, 0.2, 1) both slide-to-left;
    }

    :root::view-transition-new(root) {
        animation: 210ms cubic-bezier(0, 0, 0.2, 1) 90ms both fade-in,
        300ms cubic-bezier(0.4, 0, 0.2, 1) both slide-from-right;
    }
</style>
