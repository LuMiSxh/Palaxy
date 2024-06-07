<script lang="ts">
	import '../app.postcss';
	import {
		AppBar,
		AppShell,
		getToastStore,
		initializeStores,
		setInitialClassState,
		setModeCurrent,
		storePopup,
		TabAnchor,
		TabGroup,
		Toast
	} from '@skeletonlabs/skeleton';
	import { page } from '$app/stores';
	// Floating UI for Popups
	import { arrow, autoUpdate, computePosition, flip, offset, shift } from '@floating-ui/dom';
	// ---
	import { appDataDefault, appDataKey } from '$lib/constants';
	import { appData, toast } from '$lib/stores';
	import { Theme } from '$lib/types';
	import { generateToast, setTheme } from '$lib/functions';
	import Home from '@tabler/icons-svelte/IconHome.svelte';
	import Transform from '@tabler/icons-svelte/IconTransform.svelte';
	import Settings from '@tabler/icons-svelte/IconSettings.svelte';
	import CodeDots from '@tabler/icons-svelte/IconCodeDots.svelte';

	// Popup
	storePopup.set({ computePosition, autoUpdate, flip, shift, offset, arrow });

	// Toast
	initializeStores();
	const toastStore = getToastStore();

	toast.subscribe((t) => {
		if (!t) return;

		const toastSettings = generateToast(t);
		toastStore.trigger(toastSettings);
		toast.set(undefined);
	});

	if (window && document) {
		// Load AppData
		const appDataValue = localStorage.getItem(appDataKey);
		if (appDataValue) {
			appData.set(JSON.parse(appDataValue));
		} else {
			appData.set(appDataDefault);
		}
	}

	// Theme
	setTheme($appData.theme);

	if (window) {
		window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', (event) => {
			const newColorScheme = event.matches;

			if ($appData.theme === Theme.SYSTEM) {
				setModeCurrent(!newColorScheme);
			}
		});
	}

	const paths = [
		{
			path: '/',
			icon: Home
		},
		{
			path: '/converter',
			icon: Transform
		},
		{
			path: '/scripting',
			icon: CodeDots
		},
		{
			path: '/settings',
			icon: Settings
		}
	];
</script>

<svelte:head>
	<!-- eslint-disable-next-line svelte/no-at-html-tags -->
	{@html '<script>(' + setInitialClassState.toString() + ')();</script>'}
</svelte:head>
<Toast />
<AppShell>
	<AppBar slot="header" class="py-2">
		<a href="/" slot="lead">
			<img src="/favicon.png" class="max-h-12 select-none" alt="Logo" />
		</a>
		<h1 class="h1 select-none">
			<span
				class="bg-gradient-to-tr from-primary-500 to-fuchsia-400 bg-clip-text text-transparent box-decoration-clone"
				>Kana</span
			>
			<span
				class="bg-gradient-to-br from-fuchsia-400 to-indigo-500 bg-clip-text text-transparent box-decoration-clone"
				>Dock</span
			>
		</h1>
		<TabGroup
			slot="trail"
			justify="justify-center"
			active="variant-ghost-primary"
			hover="hover:variant-soft-primary"
			flex="flex-1 lg:flex-none"
			rounded="rounded-md"
			border=""
			class="bg-surface-100-800-token w-full"
		>
			{#each paths as path}
				<TabAnchor class="mx-0.5" href={path.path} selected={$page.url.pathname === path.path}>
					<div class="flex items-center justify-center text-center mt-0.5" slot="lead">
						<svelte:component this={path.icon} size="28" />
					</div>
				</TabAnchor>
			{/each}
		</TabGroup>
	</AppBar>
	<slot />
</AppShell>
