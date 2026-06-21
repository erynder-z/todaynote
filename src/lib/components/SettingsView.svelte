<script lang="ts">
  /**
   * Centralized settings view that aggregates folder, language, theme, and window size configurations.
   * Uses a masonry-style layout for better space utilization.
   */
  import type { Component } from 'svelte';
  import AboutSelector from './AboutSelector.svelte';
  import DefaultThreadNameSelector from './DefaultThreadNameSelector.svelte';
  import FolderSelector from './FolderSelector.svelte';
  import IdentIconStyleSelector from './IdentIconStyleSelector.svelte';
  import LanguageSelector from './LanguageSelector.svelte';
  import PurgeEmptyNotesSelector from './PurgeEmptyNotesSelector.svelte';
  import RememberSettingsSelector from './RememberSettingsSelector.svelte';
  import ShortcutSelector from './ShortcutSelector.svelte';
  import ThemeSelector from './ThemeSelector.svelte';
  import WindowSizeSelector from './WindowSizeSelector.svelte';

  let containerWidth = $state(0);

  /**
   * Number of columns based on container width.
   */
  let columnCount = $derived.by(() => {
    if (containerWidth > 1100) return 3;
    if (containerWidth > 700) return 2;
    return 1;
  });

  /**
   * List of settings components to display.
   */
  const settingsItems: Component[] = [
    FolderSelector,
    DefaultThreadNameSelector,
    LanguageSelector,
    ThemeSelector,
    IdentIconStyleSelector,
    WindowSizeSelector,
    RememberSettingsSelector,
    ShortcutSelector,
    PurgeEmptyNotesSelector,
    AboutSelector,
  ];

  /**
   * Distributes settings components into columns.
   */
  let columns = $derived.by(() => {
    const cols: Component[][] = Array.from({ length: columnCount }, () => []);
    settingsItems.forEach((item, i) => {
      cols[i % columnCount].push(item);
    });
    return cols;
  });
</script>

<div class="settings-view" bind:clientWidth={containerWidth}>
  <div class="columns-wrapper" style="--cols: {columnCount}">
    {#each columns as column}
      <div class="column-lane">
        {#each column as SettingComponent}
          <section class="settings-card">
            <SettingComponent />
          </section>
        {/each}
      </div>
    {/each}
  </div>
</div>

<style>
  .settings-view {
    width: 100%;
    height: 100%;
    padding: 1.5rem;
    overflow-y: auto;
  }

  .columns-wrapper {
    display: flex;
    gap: 1.25rem;
    align-items: flex-start;
    max-width: 1400px;
    margin: 0 auto;
  }

  .column-lane {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 1.25rem;
    min-width: 0;
  }

  .settings-card {
    background: var(--bg-surface);
    border-radius: 0.75rem;
    padding: 1.5rem;
    border: 1px solid var(--border);
    box-shadow:
      0 4px 6px -1px rgba(0, 0, 0, 0.1),
      0 2px 4px -1px rgba(0, 0, 0, 0.06);
    display: flex;
    flex-direction: column;
    height: fit-content;
    transition:
      transform 0.2s ease,
      box-shadow 0.2s ease;
  }

  @media (max-width: 768px) {
    .settings-view {
      padding: 1rem;
    }

    .columns-wrapper {
      gap: 1rem;
    }

    .column-lane {
      gap: 1rem;
    }
  }
</style>
