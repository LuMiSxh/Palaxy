<script lang="ts">
	import '../app.css';
	import { t } from 'svelte-i18n-lingui';
	import { goto } from '$app/navigation';
	import { appData, appDataKey } from '$stores/appdata.js';
	import { defaultAppData, Theme } from '$types/appdata';
	import { browser, dev } from '$app/environment';
	import { onMount } from 'svelte';
	import { keyboard } from '$lib/keyboard';
	import ActionHub from '$components/ActionHub.svelte';
	import { Badge, Toast } from 'waku/components';
	import { IconHome, IconSettings, IconTransform, IconCommand } from '@tabler/icons-svelte';
	import Dialog from '$components/Dialog.svelte';
	import { keyHint, mountGlobalKeyHintListener } from '$states/keyhint.svelte';
	import KeyHint from '$components/KeyHint.svelte';

	let { children } = $props();
	let showPalette = $state(false);

	onMount(() => {
		const unmountKeyboard = keyboard.mount();
		const unmountKeyHints = mountGlobalKeyHintListener();
		const unregisterKeyboard = keyboard.register('space', (event) => {
			event.preventDefault();
			event.stopPropagation();
			showPalette = !showPalette;
		});
		return () => {
			unmountKeyboard();
			unmountKeyHints();
			keyboard.unregister(unregisterKeyboard);
		};
	});

	// Set KeyHint
	keyHint.addKey('space', $t`ActionHub`);

	let commands = $derived([
		{
			name: $t`Home`,
			description: $t`Go to the home page`,
			icon: IconHome,
			action: () => goto('/'),
		},
		{
			name: $t`Convert`,
			description: $t`Convert your manga images into a digital format that can be read on your favorite devices`,
			icon: IconTransform,
			action: () => goto('/convert'),
		},
		{
			name: $t`Settings`,
			description: $t`Manage application preferences`,
			icon: IconSettings,
			action: () => goto('/settings'),
		},
	]);

	if (browser) {
		const appDataValue = localStorage.getItem(appDataKey);
		if (appDataValue) {
			try {
				appData.set({ ...defaultAppData, ...JSON.parse(appDataValue) });
			} catch (_err) {
				appData.set(defaultAppData);
			}
		} else {
			appData.set(defaultAppData);
		}
	}

	// Handle Theme Changes via Waku/Tailwind classes
	$effect(() => {
		if (!browser) return;
		if (
			$appData.theme === Theme.Dark ||
			($appData.theme === Theme.System && window.matchMedia('(prefers-color-scheme: dark)').matches)
		) {
			document.documentElement.classList.add('dark');
		} else {
			document.documentElement.classList.remove('dark');
		}
	});
</script>

<Toast />
<Dialog />

{#if showPalette}
	<ActionHub {commands} bind:showPalette />
{/if}

<div class="bg-surface-0 flex h-screen flex-col transition-colors duration-300">
	<main class="relative mb-10 flex-1 overflow-hidden">
		{@render children()}
	</main>

	{#if $appData.showKeyHints || $appData.mouseSupport}
		<div
			class="glass-subtle border-waku-border/50 fixed right-0 bottom-0 left-0 z-2000 flex h-10 items-center justify-center gap-4 border-t px-4"
		>
			{#if $appData.showKeyHints}
				<KeyHint />
			{/if}
			{#if $appData.mouseSupport}
				<button class="btn btn-ghost btn-sm" onclick={() => (showPalette = !showPalette)}>
					<IconCommand size={16} class="mr-2" /> ActionHub
				</button>
			{/if}
		</div>
	{/if}
</div>

{#if dev}
	<Badge variant="danger" style="subtle" class="pointer-events-none fixed right-4 bottom-12 z-50">
		Development Build
	</Badge>
{/if}
