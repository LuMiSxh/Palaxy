<script lang="ts">
	import { appData, toast } from "$lib/stores"
	import { Direction, FileFormat, InfoType, type Toast } from "$lib/types"
	import { open } from "@tauri-apps/plugin-dialog"
	import { SlideToggle } from "@skeletonlabs/skeleton"
	import { bridge } from "$lib/functions"
	import {
		IconAdjustmentsFilled,
		IconArrowAutofitContentFilled,
		IconCategoryFilled,
		IconFolderFilled,
		IconRotate2,
	} from "@tabler/icons-svelte"
	import { disableBack, disableNext, index, loading } from "$components/converter/stores"

	let targetDirectory: string | null = $appData.paths.converted
	let createDirectory: boolean = true
	let fileFormat: FileFormat = FileFormat.CBZ
	let direction: Direction = Direction.LTR

	let allFormats = Object.values(FileFormat)

	// States
	let success = false

	// Initial state
	$disableBack = false
	$disableNext = true

	// Functions
	async function select() {
		targetDirectory = (await open({
			directory: true,
			multiple: false,
		})) as string | null
	}

	function changeFileFormat() {
		// Cycle through all formats and select the next one
		fileFormat = allFormats[(allFormats.indexOf(fileFormat) + 1) % allFormats.length]
	}

	function changeReadingDirection() {
		if (fileFormat === FileFormat.EPUB)
			direction = direction === Direction.LTR ? Direction.RTL : Direction.LTR
	}

	async function convert() {
		$disableBack = true
		$loading = true
		success = false

		if (!targetDirectory) {
			toast.set({
				type: InfoType.ERROR,
				message: "Please select a target directory",
				timeout: 5000,
			} as Toast)
			$loading = false
			return
		}

		const result = await bridge("convert", {
			createDirectory,
			target: targetDirectory,
			fileFormat,
			direction,
		})

		$loading = false

		if (result) {
			success = true
		}
		$disableBack = false
	}
</script>

<div class="flex h-full w-full flex-col items-center justify-center">
	{#if $loading}
		<div></div>
	{:else if success}
		<div class="relative flex flex-col items-center justify-center">
			<div class="relative m-5 h-[50vh]">
				<img
					src="/favicon.png"
					alt="Palaxy Avatar"
					class="radial h-[50vh] object-cover backdrop-blur"
				/>
			</div>
			<button
				class="btn-large variant-filled-primary btn dark:variant-ghost-primary"
				on:click={() => {
					$index = 0
				}}
			>
				Convert Another
			</button>
		</div>
	{:else}
		<div class="grid h-full w-full grid-cols-3 grid-rows-1 gap-2">
			<div class="table-container flex items-center justify-center px-2">
				<table class="table table-interactive m-2">
					<tbody class="select-none">
						<tr on:click={select}>
							<td class="flex items-center font-bold">
								<IconFolderFilled class="mr-0.5 text-secondary-500" />
								Target Directory
							</td>
							<td>
								<code class="code whitespace-pre-wrap">
									{#if targetDirectory}
										{targetDirectory.substring(
											targetDirectory.lastIndexOf("/") + 1,
										)}
									{:else}
										Select a directory
									{/if}
								</code>
							</td>
						</tr>
						<tr>
							<td class="flex items-center font-bold">
								<IconCategoryFilled class="mr-0.5 text-secondary-500" />
								Create Folder
							</td>
							<td>
								<SlideToggle
									name="CreateFolderSlide"
									size="sm"
									active="bg-primary-400 dark:bg-primary-500"
									bind:checked={createDirectory}
								/>
							</td>
						</tr>
					</tbody>
				</table>
			</div>
			<div class="table-container flex items-center justify-center px-2">
				<table class="table table-interactive">
					<tbody class="select-none">
						<tr on:click={changeFileFormat} class="select-none">
							<td class="flex items-center font-bold">
								<IconAdjustmentsFilled class="mr-0.5 text-secondary-500" />
								File Format
							</td>
							<td>
								<code class="code whitespace-pre-wrap">
									{fileFormat}
								</code>
							</td>
						</tr>
						<tr
							on:click={changeReadingDirection}
							class="select-none {fileFormat === FileFormat.EPUB
								? ''
								: '!cursor-not-allowed opacity-50'}"
						>
							<td class="flex items-center font-bold">
								<IconArrowAutofitContentFilled class="mr-0.5 text-secondary-500" />
								Reading Direction
							</td>
							<td>
								<code class="code whitespace-pre-wrap">
									{direction}
								</code>
							</td>
						</tr>
					</tbody>
				</table>
			</div>
			<div class="flex w-full items-center justify-around p-2">
				<button
					class="btn-large variant-filled-primary btn dark:variant-soft-primary"
					disabled={!targetDirectory}
					on:click={convert}
				>
					<span class="flex items-center">
						<IconRotate2 class="mr-0.5" />
						Start Conversion
					</span>
				</button>
			</div>
		</div>
	{/if}
</div>

<style>
	.radial {
		background: radial-gradient(circle, #aa0ef5 25%, #ffd6ff 50%, transparent 70%);
	}
</style>
