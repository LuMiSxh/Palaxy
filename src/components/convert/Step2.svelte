<script lang="ts">
	import { onMount } from 'svelte';
	import { stepState } from '$states/converter.svelte';

	onMount(async () => {
		// Disable the next button, enable the previous button
		stepState.disableNext = true;
		stepState.disablePrev = false;
	})
</script>

<h1>Step 2</h1>
