<script lang="ts">
	import '../app.css';
	import { t } from 'svelte-i18n-lingui';
	import { goto } from '$app/navigation';
	import { appData, appDataKey } from '$stores/appdata.js';
	import { defaultAppData, SupportedLanguages, Theme } from '$types/appdata';
	import { browser, dev } from '$app/environment';
	import { onMount } from 'svelte';
	import { keyboard } from '$lib/keyboard';
	import ActionHub from '$components/ActionHub.svelte';
	import { Toast, toast } from 'waku/components';
	import {
		IconBrush,
		IconHome,
		IconKeyboard,
		IconLanguage,
		IconMoon,
		IconMouse,
		IconSettings,
		IconSettingsCode,
		IconSettingsQuestion,
		IconSun,
		IconTransform,
		IconHighlight,
		IconAutomation,
		IconCommand,
	} from '@tabler/icons-svelte';
	import Dialog from '$components/Dialog.svelte';
	import { openDialog } from '$states/dialog.svelte';
	import { keyHint, mountGlobalKeyHintListener } from '$states/keyhint.svelte';
	import SystemInfo from '$components/dialogs/SystemInfo.svelte';
	import KeyHint from '$components/KeyHint.svelte';
	import AutoPopulate from '$components/dialogs/AutoPopulate.svelte';

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
			description: $t`Change the settings of the application`,
			icon: IconSettings,
			subcommands: [
				{
					name: $t`Theme`,
					description: $t`Change the theme of the application`,
					icon: IconBrush,
					subcommands: [
						{
							name: $t`Light`,
							description: $t`Change the theme to light`,
							icon: IconSun,
							action: () => {
								$appData.theme = Theme.Light;
								toast($t`Theme changed to light`);
							},
						},
						{
							name: $t`Dark`,
							description: $t`Change the theme to dark`,
							icon: IconMoon,
							action: () => {
								$appData.theme = Theme.Dark;
								toast($t`Theme changed to dark`);
							},
						},
						{
							name: $t`System`,
							description: $t`Change the theme to system`,
							icon: IconAutomation,
							action: () => {
								$appData.theme = Theme.System;
								toast($t`Theme changed to system`);
							},
						},
					],
				},
				{
					name: $t`Language`,
					description: $t`Change the language of the application`,
					icon: IconLanguage,
					subcommands: [
						{
							name: $t`English`,
							description: $t`Change the language to English`,
							icon: IconLanguage,
							action: () => {
								$appData.language = SupportedLanguages.English;
								toast($t`Language changed to English`);
							},
						},
						{
							name: $t`German`,
							description: $t`Change the language to German`,
							icon: IconLanguage,
							action: () => {
								$appData.language = SupportedLanguages.German;
								toast($t`Language changed to German`);
							},
						},
					],
				},
				{
					name: $t`Auto-Populate Fields`,
					description: $t`Enable or disable the auto-population of fields in the application`,
					icon: IconHighlight,
					action: () => {
						openDialog({
							title: $t`Auto-Populate Settings`,
							content: AutoPopulate,
						});
					},
				},
				{
					name: $t`Show KeyHints`,
					description: $t`Show the key hints of the application and their actions in the current context`,
					icon: IconKeyboard,
					action: () => {
						$appData.showKeyHints = !$appData.showKeyHints;
						toast($appData.showKeyHints ? $t`KeyHints are now shown` : $t`KeyHints are now hidden`);
					},
				},
				{
					name: $t`Mouse Support`,
					description: $t`Enable or disable a button to show the ActionHub`,
					icon: IconMouse,
					action: () => {
						$appData.mouseSupport = !$appData.mouseSupport;
						toast(
							$appData.mouseSupport
								? $t`Mouse Support is now enabled`
								: $t`Mouse Support is now disabled`
						);
					},
				},
				{
					name: $t`System Information`,
					description: $t`Show the system information of the application`,
					icon: IconSettingsQuestion,
					action: () => {
						openDialog({
							title: $t`Information`,
							content: SystemInfo,
						});
					},
				},
				{
					name: $t`Reset`,
					description: $t`Reset the state of the application`,
					icon: IconSettingsCode,
					action: () => {
						openDialog({
							title: $t`Confirmation`,
							content: $t`Are you sure you want to reset the state of the application?`,
							onConfirm: () => {
								appData.set(defaultAppData);
								toast($t`Reset successfully`);
							},
							onCancel: () => {
								toast($t`Reset canceled`);
							},
						});
					},
				},
			],
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

<div class="bg-surface-0 flex min-h-screen flex-col transition-colors duration-300">
	<main class="relative flex-1 overflow-hidden">
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
	<div class="badge badge-danger pointer-events-none fixed right-4 bottom-12 z-50 opacity-50">
		Development Build
	</div>
{/if}
