<script lang="ts">
	import '../app.css';
	import { t } from 'svelte-i18n-lingui';
	import { goto } from '$app/navigation';
	import { appData, appDataKey } from '$stores/appdata.js';
	import { defaultAppData, SupportedLanguages, Theme } from '$types/appdata';
	import { setTheme } from '$lib/utils';
	import { browser, dev } from '$app/environment';
	import Toast from '$components/Toast.svelte';
	import { onMount } from 'svelte';
	import { keyboard } from '$lib/keyboard';
	import ActionHub from '$components/ActionHub.svelte';
	import {
		IconAutomation,
		IconBrush,
		IconHighlight,
		IconHome,
		IconKeyboard,
		IconLanguage,
		IconMoon,
		IconMouse,
		IconSettings,
		IconSettingsCode,
		IconSettingsQuestion,
		IconSun,
		IconTransform
	} from '@tabler/icons-svelte';
	import { addToast } from '$states/toast.svelte';
	import Dialog from '$components/Dialog.svelte';
	import { openDialog } from '$states/dialog.svelte';
	import SystemInfo from '$components/dialogs/SystemInfo.svelte';
	import KeyHint from '$components/KeyHint.svelte';
	import { keyHint } from '$states/keyhint.svelte';
	import AutoPopulate from '$components/dialogs/AutoPopulate.svelte';

	let { children } = $props();

	let showPalette = $state(false);

	onMount(() => {
		const unmountKeyboard = keyboard.mount();
		const unregisterKeyboard = keyboard.register('space', (event) => {
			event.preventDefault();
			event.stopPropagation();
			showPalette = !showPalette;
		});
		return () => {
			unmountKeyboard();
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
			action: () => goto('/')
		},
		{
			name: $t`Convert`,
			description: $t`Convert your manga images into a digital format that can be read on your favorite devices`,
			icon: IconTransform,
			action: () => goto('/convert')
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
								addToast($t`Theme changed to light`);
							}
						},
						{
							name: $t`Dark`,
							description: $t`Change the theme to dark`,
							icon: IconMoon,
							action: () => {
								$appData.theme = Theme.Dark;
								addToast($t`Theme changed to dark`);
							}
						},
						{
							name: $t`System`,
							description: $t`Change the theme to system`,
							icon: IconAutomation,
							action: () => {
								$appData.theme = Theme.System;
								addToast($t`Theme changed to system`);
							}
						}
					]
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
								addToast($t`Language changed to English`);
							}
						},
						{
							name: $t`German`,
							description: $t`Change the language to German`,
							icon: IconLanguage,
							action: () => {
								$appData.language = SupportedLanguages.German;
								addToast($t`Language changed to German`);
							}
						}
					]
				},
				{
					name: $t`Auto-Populate Fields`,
					description: $t`Enable or disable the auto-population of fields in the application`,
					icon: IconHighlight,
					action: () => {
						openDialog({
							title: $t`Auto-Populate Settings`,
							content: AutoPopulate
						});
					}
				},
				{
					name: $t`Show KeyHints`,
					description: $t`Show the key hints of the application and their actions in the current context`,
					icon: IconKeyboard,
					action: () => {
						$appData.showKeyHints = !$appData.showKeyHints;
						addToast(
							$appData.showKeyHints ? $t`KeyHints are now shown` : $t`KeyHints are now hidden`
						);
					}
				},
				{
					name: $t`Mouse Support`,
					description: $t`Enable or disable a button to show the ActionHub`,
					icon: IconMouse,
					action: () => {
						$appData.mouseSupport = !$appData.mouseSupport;
						addToast(
							$appData.mouseSupport
								? $t`Mouse Support is now enabled`
								: $t`Mouse Support is now disabled`
						);
					}
				},
				{
					name: $t`System Information`,
					description: $t`Show the system information of the application`,
					icon: IconSettingsQuestion,
					action: () => {
						openDialog({
							title: $t`Information`,
							content: SystemInfo
						});
					}
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
								addToast($t`Reset successfully`, 'success');
							},
							onCancel: () => {
								addToast($t`Reset canceled`, 'info');
							}
						});
					}
				}
			]
		}
	]);

	if (browser) {
		// Load AppData
		const appDataValue = localStorage.getItem(appDataKey);
		if (appDataValue) {
			try {
				const parsedData = JSON.parse(appDataValue);
				appData.set(parsedData);
			} catch (error) {
				console.error('Error parsing app data:', error);
				appData.set(defaultAppData);
			}
		} else {
			appData.set(defaultAppData);
		}

		window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', (event) => {
			const newColorScheme = event.matches;

			if ($appData.theme === Theme.System) {
				setTheme(newColorScheme ? Theme.Dark : Theme.Light);
			}
		});
	}
</script>

<Toast />
<Dialog />

{#if showPalette}
	<ActionHub {commands} bind:showPalette />
{/if}

<div class="bg-background dark:bg-background-dark flex h-screen w-screen flex-col overflow-hidden">
	<!-- --- -->
	<div class="flex-1 overflow-x-hidden overflow-y-hidden">
		{@render children()}
	</div>
	<!-- --- -->
	{#if $appData.showKeyHints || $appData.mouseSupport}
		<div
			class="bg-background-secondary dark:bg-background-dark-secondary z-50 flex h-8 items-center justify-center px-2"
			draggable="false"
		>
			{#if $appData.showKeyHints}
				<KeyHint />
			{/if}
			{#if $appData.mouseSupport}
				<button class="btn h-full !p-0" onclick={() => (showPalette = !showPalette)}>
					<img src="/icon.png" class="max-h-full max-w-full" alt="ActionHub" />
				</button>
			{/if}
		</div>
	{/if}
</div>

{#if dev}
	<div
		class="absolute {$appData.showKeyHints || $appData.mouseSupport
			? 'bottom-10'
			: 'bottom-5'} badge badge-error pointer-events-none right-5 text-lg! opacity-70 select-none"
		style="z-index: 9999;"
	>
		Development Build
	</div>
{/if}
