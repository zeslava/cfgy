<script lang="ts">
  import { onMount } from "svelte";
  import "../app.css";
  import { bootstrapAuth } from "$lib/auth";
  import ToastContainer from "$lib/components/ToastContainer.svelte";

  let { children } = $props();

  onMount(() => {
    bootstrapAuth();
  });
</script>

<svelte:head>
  <link rel="icon" type="image/svg+xml" href="/favicon.svg" />
</svelte:head>

<ToastContainer />
{@render children()}
