<script lang="ts">
	import Directory from '$components/converter/directory.svelte';
	import Format from '$components/converter/format.svelte';
	import Analysis from '$components/converter/analysis.svelte';
	import Volumes from '$components/converter/volumes.svelte';
	import Conversion from '$components/converter/converter.svelte';
</script>

<div class="grid grid-cols-3 grid-rows-2 gap-4 p-4">
	<Directory />
	<Volumes />
	<Analysis />
	<Format />
	<Conversion />
</div>
