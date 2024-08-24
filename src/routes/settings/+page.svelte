<script lang="ts">
	import { getName, getTauriVersion, getVersion } from "@tauri-apps/api/app"
	import { appData } from "$lib/stores"
	import { ListBox, ListBoxItem, popup, type PopupSettings, SlideToggle } from "@skeletonlabs/skeleton"
	import { Theme } from "$lib/types"
	import { open } from "@tauri-apps/plugin-dialog"
	import { version } from "$app/environment"
	import { onMount } from "svelte"

	// Popups
	const popupTheme: PopupSettings = {
		event: "click",
		target: "popupTheme",
		placement: "bottom",
	}
	const popupDefaultTargetPath: PopupSettings = {
		event: "click",
		target: "popupDefaultTargetPath",
		placement: "bottom",
	}
	const popupShowHelp: PopupSettings = {
		event: "click",
		target: "popupShowHelp",
		placement: "bottom",
	}

	onMount(async () => {
		console.log(await getTauriVersion())
	})

	// file logic
	async function selectTargetDirectory() {
		$appData.paths.converted = (await open({
			directory: true,
			multiple: false,
		})) as string | null
	}
</script>

<div
	class="grid grid-cols-2 grid-rows-1 gap-2 p-2 w-full h-full"
>
	<div class="table-container w-full h-full flex flex-col items-center justify-center p-2">
		<!-- Native Table Element -->
		<table class="table table-hover">
			<tbody>
			<tr>
				<td class="font-bold text-primary-500"> Name</td>
				<td>
					<code>
						{#await getName()}
							<div class="placeholder w-3/4 animate-pulse" />
						{:then name}
							{name}
						{:catch error}
							{error.message}
						{/await}
					</code>
				</td>
			</tr>
			<tr>
				<td class="font-bold text-primary-500"> App Version</td>
				<td>
					<code>
						{#await getVersion()}
							<div class="placeholder w-3/4 animate-pulse" />
						{:then name}
							{name}
						{:catch error}
							{error.message}
						{/await}
					</code>
				</td>
			</tr>
			<tr>
				<td class="font-bold text-primary-500"> Tauri Version</td>
				<td>
					<code>
						{#await getTauriVersion()}
							<div class="placeholder w-3/4 animate-pulse" />
						{:then name}
							{name}
						{:catch error}
							{error.message}
						{/await}
					</code>
				</td>
			</tr>
			<tr>
				<td class="font-bold text-primary-500"> Svelte Version Hash</td>
				<td>
					<code>
						{version}
					</code>
				</td>
			</tr>
			</tbody>
		</table>
	</div>
	<div class="table-container w-full h-full flex flex-col items-center justify-center p-2">
		<!-- Native Table Element -->
		<table class="table table-interactive">
			<tbody>
			<tr use:popup={popupTheme}>
				<td class="font-bold text-primary-500"> Theme</td>
				<td>
					<code>
						{$appData.theme}
					</code>
				</td>
			</tr>
			<tr use:popup={popupDefaultTargetPath}>
				<td class="font-bold text-primary-500"> Default Target Path</td>
				<td>
					<code>
						{$appData.paths.converted
							? $appData.paths.converted.substring(
								$appData.paths.converted.lastIndexOf("/") + 1,
							)
							: "Not Set"}
					</code>
				</td>
			</tr>
			<tr use:popup={popupShowHelp}>
				<td class="font-bold text-primary-500"> Show help</td>
				<td>
					<code>
						{$appData.popups.help ? "Yes" : "No"}
					</code>
				</td>
			</tr>
			</tbody>
		</table>
	</div>
</div>
<!-- popups -->
<div class="card variant-soft-tertiary w-36 p-4 shadow-2xl" data-popup="popupTheme">
	<ListBox class="variant-soft-surface">
		<ListBoxItem bind:group={$appData.theme} name="medium" value={Theme.SYSTEM}
		>System
		</ListBoxItem
		>
		<ListBoxItem bind:group={$appData.theme} name="medium" value={Theme.DARK}>Dark</ListBoxItem>
		<ListBoxItem bind:group={$appData.theme} name="medium" value={Theme.LIGHT}
		>Light
		</ListBoxItem
		>
	</ListBox>
	<div class="variant-soft-tertiary arrow" />
</div>

<div class="card variant-soft-tertiary w-36 p-4 shadow-2xl" data-popup="popupDefaultTargetPath">
	<div class="grid grid-cols-1 grid-rows-2 gap-2">
		<button class="variant-filled-primary btn" on:click={selectTargetDirectory}> Select</button>
		<button class="variant-filled-secondary btn" on:click={() => ($appData.paths.converted = null)}>
			Remove
		</button>
	</div>
	<div class="variant-soft-tertiary arrow" />
</div>

<div class="card variant-soft-tertiary w-36 p-4 shadow-2xl" data-popup="popupShowHelp">
	<div class="flex items-center justify-center">
		<SlideToggle name="Show Popup" bind:checked={$appData.popups.help} active="bg-primary-500" />
	</div>
	<div class="variant-soft-tertiary arrow" />
</div>
