<script lang="ts">
	let { status }: { status: string } = $props();
</script>

<p class="flex items-center gap-2 text-sm">
	<span class="status status-success" aria-hidden="true"></span>
	API status: {status}
</p>
