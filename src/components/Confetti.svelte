<script lang="ts">
	import { onDestroy, onMount } from 'svelte';
	import { fade } from 'svelte/transition';

	interface Props {
		count: number | undefined;
		duration: number | null;
		autoStart: boolean | undefined;
	}

	let { count = 100, duration = null, autoStart = true }: Props = $props();

	let isActive = $state(false);
	let confettiItems: { id: string; props: Record<string, string> }[] = $state([]);
	let spawnInterval: ReturnType<typeof setInterval> | null = null;

	function generateConfettiProps() {
		return {
			'--random-x': `${Math.random() * 100}vw`,
			'--random-y': `-${Math.random() * 20}vh`,
			'--random-delay': `${Math.random() * 0.5}s`,
			'--random-size': `${Math.random() * 0.7 + 0.3}rem`,
			'--random-rotation': `${Math.random() * 360}deg`,
			'--random-color': `hsl(${Math.random() * 360}, 70%, 60%)`,
			'--animation-duration': `${Math.random() * 3 + 2}s`,
		};
	}

	function createInitialConfetti() {
		confettiItems = Array(count)
			.fill(0)
			.map(() => ({
				id: crypto.randomUUID(),
				props: generateConfettiProps(),
			}));
	}

	function setupContinuousSpawning() {
		if (duration === null && isActive) {
			// Create new confetti every 200ms
			spawnInterval = setInterval(() => {
				if (!isActive) return;

				// Add a few new confetti particles
				const newConfetti = Array(5)
					.fill(0)
					.map(() => ({
						id: crypto.randomUUID(),
						props: generateConfettiProps(),
					}));

				confettiItems = [...confettiItems, ...newConfetti];

				// Limit the total number of active confetti particles
				if (confettiItems.length > 300) {
					confettiItems = confettiItems.slice(-300);
				}
			}, 200);
		}
	}

	// Start the confetti explosion
	export function start() {
		isActive = true;
		createInitialConfetti();
		setupContinuousSpawning();

		// Automatically clean up after animation ends
		if (duration !== null && duration > 0) {
			setTimeout(() => {
				stop();
			}, duration);
		}
	}

	// Stop the confetti early if needed
	export function stop() {
		isActive = false;
		if (spawnInterval) {
			clearInterval(spawnInterval);
			spawnInterval = null;
		}
		confettiItems = [];
	}

	onMount(() => {
		if (autoStart) {
			start();
		}
	});

	onDestroy(() => {
		if (spawnInterval) {
			clearInterval(spawnInterval);
		}
	});
</script>

{#if isActive}
	<div class="confetti-container" transition:fade={{ duration: 800 }}>
		{#each confettiItems as item (item.id)}
			<div
				class="confetti"
				style={Object.entries(item.props)
					.map(([k, v]) => `${k}: ${v}`)
					.join(';')}
			></div>
		{/each}
	</div>
{/if}

<style>
	.confetti-container {
		position: fixed;
		top: 0;
		left: 0;
		width: 100vw;
		height: 100vh;
		pointer-events: none;
		z-index: 100;
		overflow: hidden;
	}

	.confetti {
		position: absolute;
		top: -20px;
		left: var(--random-x, 50%);
		width: var(--random-size, 0.5rem);
		height: var(--random-size, 0.5rem);
		background-color: var(--random-color, #709bda);
		opacity: 0.8;
		border-radius: 2px;
		transform: rotate(var(--random-rotation, 0deg));
		animation: confetti-fall var(--animation-duration, 3s) ease-in forwards var(--random-delay, 0s);
	}

	/* Create different shapes of confetti */
	.confetti:nth-child(3n) {
		border-radius: 50%;
	}

	.confetti:nth-child(3n + 1) {
		clip-path: polygon(50% 0%, 0% 100%, 100% 100%);
	}

	@keyframes confetti-fall {
		0% {
			transform: translateY(var(--random-y, 0)) rotate(0);
			opacity: 1;
		}
		80% {
			opacity: 1;
		}
		100% {
			transform: translateY(100vh) rotate(720deg);
			opacity: 0;
		}
	}
</style>
