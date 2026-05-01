<script lang="ts">
  /**
   * Application entry point. Handles initial bootstrapping and renders the main UI structure.
   */
  import { onMount } from 'svelte';
  import {
    initializeApp,
    lang,
    locale,
    MainView,
    Navigation,
    PopupManager,
  } from '$lib';

  /**
   * Bootstraps the app by fetching initial state from the backend.
   */
  onMount(async () => {
    await initializeApp();
  });
</script>

<svelte:body use:lang={$locale} />

<main class="container">
  <Navigation />
  <div class="content-area">
    <MainView />
    <PopupManager />
  </div>
</main>

<style>
  @font-face {
    font-family: SUSE Mono;
    src: url(../assets/SUSEMono-VariableFont_wght.ttf) format('truetype');
  }

  :global(:root) {
    box-sizing: border-box;
    --font-mono: 'SUSE Mono', monospace;
    font-family: var(--font-mono);
    font-weight: 400;
    background-color: var(--bg-base);
    color: var(--text-main);
    line-height: 1.6;
    -webkit-font-smoothing: antialiased;
  }

  :global(*),
  :global(*::before),
  :global(*::after) {
    box-sizing: inherit;
    margin: 0;
    padding: 0;
  }
  .container {
    height: 100dvh;
    width: 100vw;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: row;
    overflow: hidden;
    background-color: var(--bg-base);
    transition:
      background-color 0.3s,
      color 0.3s;
  }

  .content-area {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    position: relative;
    overflow: hidden;
  }

  @media (max-width: 1024px) {
    .container {
      flex-direction: column;
    }
  }

  @media (max-width: 480px) {
    .content-area {
      padding: 0;
    }
  }

  :global(::-webkit-scrollbar) {
    width: 8px;
    height: 8px;
  }

  :global(::-webkit-scrollbar-track) {
    background: var(--bg-base);
    border-radius: 4px;
  }

  :global(::-webkit-scrollbar-thumb) {
    background: color-mix(in srgb, var(--text-muted), transparent 60%);
    border-radius: 4px;
    border: 2px solid var(--bg-base);
  }

  :global(::-webkit-scrollbar-thumb:hover) {
    background: color-mix(in srgb, var(--text-muted), transparent 40%);
  }

  :global(html) {
    scrollbar-width: thin;
    scrollbar-color: color-mix(in srgb, var(--text-muted), transparent 60%)
      var(--bg-base);
  }

  :global(*:hover) {
    scrollbar-color: color-mix(in srgb, var(--text-muted), transparent 40%)
      var(--bg-base);
  }
</style>
