<script lang="ts">
	import '../app.css';
	import { t } from 'svelte-i18n-lingui';
	import { goto } from '$app/navigation';
	import { appData, appDataKey } from '$stores/appdata.js';
	import { defaultAppData, SupportedLanguages, Theme } from '$types/appdata';
	import { setTheme } from '$lib/utils';
	import { browser } from '$app/environment';
	import Toast from '$components/Toast.svelte';
	import { onMount } from 'svelte';
	import { keyboard } from '$lib/keyboard';
	import CommandPalette from '$components/CommandPalette.svelte';
	import { IconHome, IconLanguage, IconSearch, IconSettings, IconTransform, IconUsers } from '@tabler/icons-svelte';
	import { addToast } from '$states/toast.svelte';
	import Dialog from '$components/Dialog.svelte';
	import { openDialog } from '$states/dialog.svelte';

	let { children } = $props();

	let showPalette = $state(false);

	onMount(() => {
		const unmountKeyboard = keyboard.mount();
		const unregisterKeyboard = keyboard.register(
			'space',
			(event) => {
				event.preventDefault();
				event.stopPropagation();
				showPalette = !showPalette;
			}
		);
		return () => {
			unmountKeyboard();
			keyboard.unregister(unregisterKeyboard);
		};
	});

	let commands = $derived([
		{
			name: $t`Home`,
			description: $t`Go back to the home page`,
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
			name: $t`Search`,
			description: $t`Search for your favorite manga series and chapters from various sources`,
			icon: IconSearch,
			action: () => goto('/search')
		},
		{
			name: $t`Agents`,
			description: $t`Manage your agents and their settings for better search results`,
			icon: IconUsers,
			action: () => goto('/agents')
		},
		{
			name: $t`Settings`,
			description: $t`Change the settings of the application`,
			icon: IconSettings,
			subcommands: [
				{
					name: $t`Theme`,
					description: $t`Change the theme of the application`,
					icon: IconSettings,
					subcommands: [
						{
							name: $t`Light`,
							description: $t`Change the theme to light`,
							icon: IconSettings,
							action: () => {
								setTheme(Theme.Light);
								addToast($t`Theme changed to light`);
							}
						},
						{
							name: $t`Dark`,
							description: $t`Change the theme to dark`,
							icon: IconSettings,
							action: () => {
								setTheme(Theme.Dark);
								addToast($t`Theme changed to dark`);
							}
						},
						{
							name: $t`System`,
							description: $t`Change the theme to system`,
							icon: IconSettings,
							action: () => {
								setTheme(Theme.System);
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
						},
						{
							name: $t`Japanese`,
							description: $t`Change the language to Japanese`,
							icon: IconLanguage,
							action: () => {
								$appData.language = SupportedLanguages.Japanese;
								addToast($t`Language changed to Japanese`);
							}
						}
					]
				},
				{
					name: $t`Reset`,
					description: $t`Reset the settings of the application`,
					icon: IconSettings,
					action: () => {
						openDialog({
							title: $t`Confirmation`,
							content: $t`Are you sure you want to reset the settings of the application?`,
							onConfirm: () => {
								appData.set(defaultAppData);
								addToast($t`Reset successfull`, 'success');
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
			appData.set(JSON.parse(appDataValue));
		} else {
			appData.set(defaultAppData);
		}

		window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', (event) => {
			const newColorScheme = event.matches;

			if ($appData.theme === Theme.System) {
				setTheme(newColorScheme ? Theme.Dark : Theme.Light);
			}
		});

		// Set dark / light mode
		setTheme($appData.theme);
	}
</script>

<Toast />
<Dialog />

{#if showPalette}
	<CommandPalette {commands} bind:showPalette={showPalette} />
{/if}

<div class="h-screen w-screen flex flex-col bg-background dark:bg-background-dark overflow-hidden">
	<!-- --- -->
	<div class="flex-1 overflow-y-hidden">
		{@render children()}
	</div>
	<!-- --- -->
	<div class="h-8 z-50 flex flex-col bg-background-secondary dark:bg-background-dark-secondary">
		Bottom
	</div>
</div>
