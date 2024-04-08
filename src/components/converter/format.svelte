<script lang="ts">
	import { converter } from '$lib/stores';
	import FileInfo from '@tabler/icons-svelte/IconFileInfo.svelte';
	import LetterSpacing from '@tabler/icons-svelte/IconLetterSpacing.svelte';
	import { Direction, FileFormat } from '$lib/types';
	import { ListBox, ListBoxItem, type PopupSettings } from '@skeletonlabs/skeleton';
	import { popup } from '@skeletonlabs/skeleton';

	const popupFormat: PopupSettings = {
		event: 'click',
		target: 'popupFormat',
		placement: 'top'
	};

	function changeReadingDirection() {
		if ($converter.format === FileFormat.EPUB)
			$converter.direction = $converter.direction === Direction.LTR ? Direction.RTL : Direction.LTR;
	}
</script>


<div class="card">
	<header class="card-header">
		<h2 class="text-xl font-bold">
			Format
		</h2>
	</header>
	<section class="table-container p-4">
		<!-- Native Table Element -->
		<table class="table table-interactive">
			<tbody>
			<tr use:popup={popupFormat} class="select-none">
				<td class="font-bold text-secondary-500 flex items-center">
					<FileInfo class="mr-0.5"/>
					File Format
				</td>
				<td>
					<code class="code whitespace-pre-wrap">
						{$converter.format}
					</code>
				</td>
			</tr>
			<tr on:click={changeReadingDirection} class="select-none {$converter.format === FileFormat.EPUB? '' : 'opacity-50 !cursor-not-allowed'}">
				<td class="font-bold text-secondary-500 flex items-center">
					<LetterSpacing class="mr-0.5"/>
					Reading Direction
				</td>
				<td>
					<code class="code whitespace-pre-wrap">
						{$converter.direction}
					</code>
				</td>
			</tr>
			</tbody>
		</table>
	</section>
</div>
<!-- popups -->
<div class="card variant-glass-surface p-4 shadow-xl" data-popup="popupFormat">
	<ListBox>
		{#each Object.values(FileFormat) as format}
			<ListBoxItem bind:group={$converter.format} name="medium" value={format}>
				{format}
			</ListBoxItem>
		{/each}
	</ListBox>
	<div class="arrow variant-glass-surface" />
</div>
