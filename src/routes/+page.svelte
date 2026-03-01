<script lang="ts">
  import { onMount } from 'svelte';
  import {
    initializeApp,
    lang,
    locale,
    MainView,
    Navigation,
    PopupManager,
  } from '$lib';

  onMount(async () => {
    await initializeApp();
  });
</script>

<svelte:body use:lang={$locale} />

<main class="container">
  <MainView />
  <Navigation />
  <PopupManager />
</main>

<style>
  :root {
    font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
    background-color: var(--bg-base);
    color: var(--text-main);
    transition:
      background-color 0.3s,
      color 0.3s;
  }

  .container {
    max-width: 800px;
    margin: 0 auto;
    padding: 2rem 1rem;
    min-height: 100vh;
    display: flex;
    flex-direction: column;
  }
</style>
