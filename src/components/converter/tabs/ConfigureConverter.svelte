<script lang="ts">
	import type { Writable } from 'svelte/store';
	import { BundlerFlag } from '$components/converter/types';
	import { appData, converterTab, tabDisableBack, tabDisableNext } from '$lib/stores';
	import { Direction, FileFormat } from '$lib/types';
	import { open } from '@tauri-apps/api/dialog';
	import { ProgressRadial, SlideToggle } from '@skeletonlabs/skeleton';
	import DatabaseImport from '@tabler/icons-svelte/IconDatabaseImport.svelte';
	import Folder from '@tabler/icons-svelte/IconFolder.svelte';
	import FileInfo from '@tabler/icons-svelte/IconFileInfo.svelte';
	import LetterSpacing from '@tabler/icons-svelte/IconLetterSpacing.svelte';
	import Rotate_2 from '@tabler/icons-svelte/IconRotate2.svelte';
	import { bridge } from '$lib/functions';
	import SquareX from '@tabler/icons-svelte/IconSquareX.svelte';
	import SquareCheck from '@tabler/icons-svelte/IconSquareCheck.svelte';

	export let bundler: Writable<BundlerFlag | null>;
	export let sourceManga: Writable<string | null>;
	export let cpv: Writable<Array<number> | null>;

	let targetDirectory: string | null = $appData.paths.converted;
	let createDirectory: boolean = true;
	let fileFormat: FileFormat = FileFormat.CBZ;
	let direction: Direction = Direction.LTR;

	let allFormats = Object.values(FileFormat);

	// States
	let loading = false;
	let error = false;
	let success = true;

	// Initial state
	$tabDisableBack = false;
	$tabDisableNext = true;

	// Functions
	async function select() {
		targetDirectory = (await open({
			directory: true,
			multiple: false
		})) as string | null;
	}

	function changeFileFormat() {
		// Cycle through all formats and select the next one
		fileFormat = allFormats[(allFormats.indexOf(fileFormat) + 1) % allFormats.length];
	}

	function changeReadingDirection() {
		if (fileFormat === FileFormat.EPUB)
			direction = direction === Direction.LTR ? Direction.RTL : Direction.LTR;
	}

	async function convert() {
		$tabDisableBack = true;
		loading = true;
		error = false;
		success = false;

		if (!targetDirectory || !sourceManga) {
			error = true;
			loading = false;
			return;
		}

		const result = await bridge('flow_convert', {
			sourceDirectory: $sourceManga,
			targetDirectory,
			chaptersPerVolume: $cpv,
			bundlerFlag: $bundler,
			createDirectory,
			fileFormat,
			direction
		});

		loading = false;

		if (result) {
			success = true;
		} else {
			error = true;
		}
		$tabDisableBack = false;
	}
</script>

<div class="w-full min-h-full flex items-center justify-center flex-col">
	{#if loading}
		<div class="h-full w-full flex items-center justify-center animate-pulse">
			<ProgressRadial value={undefined} strokeLinecap="round" meter="stroke-secondary-500" />
		</div>
	{:else if error}
		<div class="h-full w-full flex flex-col items-center justify-center">
			<SquareX size="128" class="text-error-600 dark:text-error-600" />
			<h3 class="h3 text-error-600 dark:text-error-600">Oops! Something went wrong!</h3>
		</div>
	{:else if success}
		<div class="h-full w-full flex flex-col items-center justify-center">
			<SquareCheck size="128" class="text-success-500" />
			<button
				class="btn btn-large variant-filled-success dark:variant-soft-success"
				on:click={() => {
					$converterTab = 0;
				}}
			>
				Convert Another
			</button>
		</div>
	{:else}
		<div class="flex w-full h-full items-start justify-around">
			<div class="table-container w-1/3">
				<table class="table table-interactive">
					<thead>
						<tr>
							<th></th>
							<th></th>
						</tr>
					</thead>
					<tbody class="select-none">
						<tr on:click={select}>
							<td class="font-bold text-secondary-500 flex items-center">
								<DatabaseImport class="mr-0.5" />
								Target Directory
							</td>
							<td>
								<code class="code whitespace-pre-wrap">
									{#if targetDirectory}
										{targetDirectory.substring(targetDirectory.lastIndexOf('/') + 1)}
									{:else}
										Select a directory
									{/if}
								</code>
							</td>
						</tr>
						<tr>
							<td class="font-bold text-secondary-500 flex items-center">
								<Folder class="mr-0.5" />
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
			<div class="table-container w-1/3">
				<table class="table table-interactive">
					<thead>
						<tr>
							<th></th>
							<th></th>
						</tr>
					</thead>
					<tbody class="select-none">
						<tr on:click={changeFileFormat} class="select-none">
							<td class="font-bold text-secondary-500 flex items-center">
								<FileInfo class="mr-0.5" />
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
								: 'opacity-50 !cursor-not-allowed'}"
						>
							<td class="font-bold text-secondary-500 flex items-center">
								<LetterSpacing class="mr-0.5" />
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
			<div class="w-1/4 min-h-40 flex items-center justify-around">
				<button
					class="btn btn-large variant-filled-primary dark:variant-soft-primary"
					disabled={!targetDirectory}
					on:click={convert}
				>
					<span class="flex items-center">
						<Rotate_2 class="mr-0.5" />
						Start Conversion
					</span>
				</button>
			</div>
		</div>
	{/if}
</div>
