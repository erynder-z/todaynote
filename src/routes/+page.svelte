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
  @font-face {
    font-family: SUSE Mono;
    src: url(../assets/SUSEMono-VariableFont_wght.ttf) format('truetype');
  }

  :root {
    font-family: 'SUSE Mono', monospace;
    font-weight: 600;
    background-color: var(--bg-base);
    color: var(--text-main);
    transition:
      background-color 0.3s,
      color 0.3s;
    line-height: 1.6;
    -webkit-font-smoothing: antialiased;
  }

  .container {
    max-width: 50rem;
    margin: 0 auto;
    padding: 2rem 1.5rem;
    min-height: 100vh;
    display: flex;
    flex-direction: column;
  }
</style>
