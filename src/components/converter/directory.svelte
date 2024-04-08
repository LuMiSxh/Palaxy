<script lang="ts">
	import { converter } from '$lib/stores';
	import { open } from '@tauri-apps/api/dialog';
	import DatabaseExport from '@tabler/icons-svelte/IconDatabaseExport.svelte';
	import DatabaseImport from '@tabler/icons-svelte/IconDatabaseImport.svelte';

	// I/O functions
	async function selectSourceDirectory() {
		$converter.sourceDirectory = await open({
			directory: true,
			multiple: false
		}) as string | null;
	}

	async function selectTargetDirectory() {
		$converter.targetDirectory = await open({
			directory: true,
			multiple: false
		}) as string | null;
	}
</script>

<div class="card">
	<header class="card-header">
		<h2 class="text-xl font-bold">
			Directories
		</h2>
	</header>
	<section class="table-container p-4">
			<!-- Native Table Element -->
			<table class="table table-interactive">
				<tbody>
				<tr on:click={selectSourceDirectory} class="select-none">
					<td class="font-bold text-secondary-500 flex items-center">
						<DatabaseExport class="mr-0.5"/>
						Source Directory
					</td>
					<td>
						<code class="code whitespace-pre-wrap">
							{#if $converter.sourceDirectory}
								{$converter.sourceDirectory.substring($converter.sourceDirectory.lastIndexOf('/') + 1)}
							{:else}
								Select a directory
							{/if}
						</code>
					</td>
				</tr>
				<tr on:click={selectTargetDirectory} class="select-none">
					<td class="font-bold text-secondary-500 flex items-center">
						<DatabaseImport class="mr-0.5"/>
						Target Directory
					</td>
					<td>
						<code class="code whitespace-pre-wrap">
							{#if $converter.targetDirectory}
								{$converter.targetDirectory.substring($converter.targetDirectory.lastIndexOf('/') + 1)}
							{:else}
								Select a directory
							{/if}
						</code>
					</td>
				</tr>
				</tbody>
			</table>
	</section>
</div>

