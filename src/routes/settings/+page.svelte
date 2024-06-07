<script lang="ts">
	import { getName, getTauriVersion, getVersion } from '@tauri-apps/api/app';
	import { appData } from '$lib/stores';
	import {
		ListBox,
		ListBoxItem,
		popup,
		type PopupSettings,
		SlideToggle
	} from '@skeletonlabs/skeleton';
	import { Theme } from '$lib/types';
	import { open } from '@tauri-apps/api/dialog';
	import { version } from '$app/environment';

	// Popups
	const popupTheme: PopupSettings = {
		event: 'click',
		target: 'popupTheme',
		placement: 'bottom'
	};
	const popupDefaultTargetPath: PopupSettings = {
		event: 'click',
		target: 'popupDefaultTargetPath',
		placement: 'bottom'
	};
	const popupShowHelp: PopupSettings = {
		event: 'click',
		target: 'popupShowHelp',
		placement: 'bottom'
	};
	const popupShowInfo: PopupSettings = {
		event: 'click',
		target: 'popupShowInfo',
		placement: 'bottom'
	};

	// file logic
	async function selectTargetDirectory() {
		$appData.paths.converted = (await open({
			directory: true,
			multiple: false
		})) as string | null;
	}
</script>

<div class="grid grid-cols-3 grid-rows-1 gap-4 p-4">
	<div class="card">
		<header class="card-header">
			<h2 class="text-xl font-bold">App Info</h2>
		</header>
		<section class="p-4">
			<div class="table-container">
				<!-- Native Table Element -->
				<table class="table table-hover">
					<tbody>
						<tr>
							<td class="font-bold text-secondary-500"> Name </td>
							<td>
								<code>
									{#await getName()}
										<div class="placeholder animate-pulse w-3/4" />
									{:then name}
										{name}
									{:catch error}
										{error.message}
									{/await}
								</code>
							</td>
						</tr>
						<tr>
							<td class="font-bold text-secondary-500"> App Version </td>
							<td>
								<code>
									{#await getVersion()}
										<div class="placeholder animate-pulse w-3/4" />
									{:then name}
										{name}
									{:catch error}
										{error.message}
									{/await}
								</code>
							</td>
						</tr>
						<tr>
							<td class="font-bold text-secondary-500"> Tauri Version </td>
							<td>
								<code>
									{#await getTauriVersion()}
										<div class="placeholder animate-pulse w-3/4" />
									{:then name}
										{name}
									{:catch error}
										{error.message}
									{/await}
								</code>
							</td>
						</tr>
						<tr>
							<td class="font-bold text-secondary-500"> Svelte Version Hash </td>
							<td>
								<code>
									{version}
								</code>
							</td>
						</tr>
					</tbody>
				</table>
			</div>
		</section>
	</div>
	<div class="card">
		<header class="card-header">
			<h2 class="text-xl font-bold">App Data</h2>
		</header>
		<section class="p-4">
			<div class="table-container">
				<!-- Native Table Element -->
				<table class="table table-interactive">
					<tbody>
						<tr use:popup={popupTheme}>
							<td class="font-bold text-secondary-500"> Theme </td>
							<td>
								<code>
									{$appData.theme}
								</code>
							</td>
						</tr>
						<tr use:popup={popupDefaultTargetPath}>
							<td class="font-bold text-secondary-500"> Default Target Path </td>
							<td>
								<code>
									{$appData.paths.converted
										? $appData.paths.converted.substring(
												$appData.paths.converted.lastIndexOf('/') + 1
											)
										: 'Not Set'}
								</code>
							</td>
						</tr>
						<tr use:popup={popupShowHelp}>
							<td class="font-bold text-secondary-500"> Show help </td>
							<td>
								<code>
									{$appData.popups.help ? 'Yes' : 'No'}
								</code>
							</td>
						</tr>
						<tr use:popup={popupShowInfo}>
							<td class="font-bold text-secondary-500"> Show info </td>
							<td>
								<code>
									{$appData.popups.info ? 'Yes' : 'No'}
								</code>
							</td>
						</tr>
					</tbody>
				</table>
			</div>
		</section>
	</div>
	<div class="card">
		<header class="card-header">
			<h2 class="text-xl font-bold">Scripting</h2>
		</header>
		<section class="p-4">Placeholder</section>
	</div>
</div>
<!-- popups -->
<div class="card variant-glass-surface p-4 w-36 shadow-xl" data-popup="popupTheme">
	<ListBox>
		<ListBoxItem bind:group={$appData.theme} name="medium" value={Theme.SYSTEM}>System</ListBoxItem>
		<ListBoxItem bind:group={$appData.theme} name="medium" value={Theme.DARK}>Dark</ListBoxItem>
		<ListBoxItem bind:group={$appData.theme} name="medium" value={Theme.LIGHT}>Light</ListBoxItem>
	</ListBox>
	<div class="arrow variant-glass-surface" />
</div>

<div class="card variant-glass-surface p-4 w-36 shadow-xl" data-popup="popupDefaultTargetPath">
	<div class="grid grid-cols-1 grid-rows-2 gap-2">
		<button class="btn variant-filled-success" on:click={selectTargetDirectory}> Select </button>
		<button class="btn variant-filled-error" on:click={() => ($appData.paths.converted = null)}>
			Remove
		</button>
	</div>
	<div class="arrow variant-glass-surface" />
</div>

<div class="card variant-glass-surface p-4 w-36 shadow-xl" data-popup="popupShowHelp">
	<div class="flex items-center justify-center">
		<SlideToggle name="Show Popup" bind:checked={$appData.popups.help} />
	</div>
	<div class="arrow variant-glass-surface" />
</div>

<div class="card variant-glass-surface p-4 w-36 shadow-xl" data-popup="popupShowInfo">
	<div class="flex items-center justify-center">
		<SlideToggle name="Show Popup" bind:checked={$appData.popups.info} />
	</div>
	<div class="arrow variant-glass-surface" />
</div>
