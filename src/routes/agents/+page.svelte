<script lang="ts">
	export let data;
	const agents = data.agents;
</script>

<h1>Agents</h1>


{#if agents.length > 0}
	<div class="grid grid-cols-3">
		{#each agents as agent}
			<div class="card bg-surface-400-500-token m-2">
				<section class="card-header flex justify-between mr-3 pb-2">
					<a href={agent.url} target="_blank" class="h3">{agent.name}</a>
					{#if agent.icon}
						<img src={agent.icon} class="w-9 h-9 rounded-full border border-primary-500/50" alt={agent.name} />
					{:else}
						<div class="bg-surface-400/50 w-9 h-9 rounded-full border border-primary-500/50"/>
					{/if}
				</section>
				<div class="flex flex-wrap p-2 justify-start">
					{#each agent.tags as tag}
						<span
							class="chip variant-ghost-secondary m-1"
							class:variant-ghost-success={tag === "Supported"}
							class:variant-ghost-error={tag === "Unsupported"}
							class:variant-ghost-warning={tag === "Experimental"}
						>
							{tag}
						</span>
					{/each}
				</div>
			</div>
		{/each}
	</div>
{:else}
	<p>ERR</p>
{/if}
