<script lang="ts">
	import './layout.css';
	import '../app.css';
	import { onMount } from 'svelte';

	onMount(() => {
		document.documentElement.classList.add('dark');
	});
</script>

<slot></slot>
