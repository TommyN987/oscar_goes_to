<script lang="ts">
	import ApiStatus from './ApiStatus.svelte';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();
</script>

<svelte:head>
	<title>The Gold Ballot</title>
</svelte:head>

<main class="hero min-h-screen">
	<div class="hero-content flex-col text-center">
		<h1 class="text-4xl font-semibold tracking-widest sm:text-6xl">THE GOLD BALLOT</h1>
		<p class="text-base-content/70">Guess the Oscars with Tommy</p>
		<ApiStatus status={data.liveness.status} />
	</div>
</main>
