<script lang="ts">
	import { converter } from '$lib/stores';
	import Minus from '@tabler/icons-svelte/IconMinus.svelte';
	import Plus from '@tabler/icons-svelte/IconPlus.svelte';

	let tempVolume = 1;
</script>

<div class="card row-span-2">
	<section class="p-4 table-container max-h-[28rem] overflow-y-scroll no-scrollbar">
		<!-- Native Table Element -->
		<table class="table {$converter.volumes.length !== 0? 'table-interactive' : 'table-hover'}">
			<thead>
			<tr>
				<th>
					Volume
				</th>
				<th>
					Chapters
				</th>
			</tr>
			</thead>
			<tbody>
			{#if $converter.volumes.length !== 0}
				<!-- eslint-disable-next-line @typescript-eslint/no-unused-vars -->
				{#each $converter.volumes as _, index}
					<tr class="select-none">
						<td class="font-bold text-secondary-500">
							{index + 1}
						</td>
						<td class="flex items-center justify-evenly">
							<input
								class="input w-1/2 h-7"
								type="number"
								min="0"
								max="999"
								disabled={$converter.analysis.running || $converter.running}
								bind:value={$converter.volumes[index]}
							/>
							<button
								class="btn variant-filled-error dark:variant-soft-error h-7"
								on:click={() => {$converter.volumes.splice(index, 1); $converter.volumes = $converter.volumes;}}
								disabled={$converter.analysis.running || $converter.running}
							>
								<Minus size={18}/>
							</button>
						</td>
					</tr>
				{/each}
			{/if}
			<tr class="select-none animate-pulse">
				<td class="font-bold text-primary-500">
					{$converter.volumes.length + 1}
				</td>
				<td class="flex items-center justify-evenly">
					<input
						class="input w-1/2 h-7"
						type="number"
						min="0"
						max="999"
						disabled={$converter.analysis.running || $converter.running}
						bind:value={tempVolume}
					/>
					<button
						class="btn variant-filled-success dark:variant-soft-success h-7"
						on:click={() => {$converter.volumes.push(tempVolume); $converter.volumes = $converter.volumes;}}
						disabled={$converter.analysis.running || $converter.running}
					>
						<Plus size={18}/>
					</button>
				</td>
			</tr>
			</tbody>
		</table>
	</section>
</div>

<style lang="postcss">
    /* Hide scrollbar for Chrome, Safari and Opera */
    .no-scrollbar::-webkit-scrollbar {
        display: none;
    }

    /* Hide scrollbar for IE, Edge and Firefox */
    .no-scrollbar {
        -ms-overflow-style: none;  /* IE and Edge */
        scrollbar-width: none;  /* Firefox */
    }
</style>
