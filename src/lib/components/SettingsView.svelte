<script lang="ts">
  /**
   * Centralized settings view that aggregates folder, language, theme, and window size configurations.
   */

  import DefaultThreadNameSelector from './DefaultThreadNameSelector.svelte';
  import FolderSelector from './FolderSelector.svelte';
  import LanguageSelector from './LanguageSelector.svelte';
  import RememberSettingsSelector from './RememberSettingsSelector.svelte';
  import ShortcutSelector from './ShortcutSelector.svelte';
  import ThemeSelector from './ThemeSelector.svelte';
  import WindowSizeSelector from './WindowSizeSelector.svelte';
</script>

<div class="settings-view">
  <FolderSelector />
  <div class="divider"></div>
  <DefaultThreadNameSelector />
  <div class="divider"></div>
  <LanguageSelector />
  <div class="divider"></div>
  <ThemeSelector />
  <div class="divider"></div>
  <WindowSizeSelector />
  <div class="divider"></div>
  <RememberSettingsSelector />
  <div class="divider"></div>
  <ShortcutSelector />
</div>

<style>
  .settings-view {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1.25rem;
    width: 100%;
    height: 100%;
    padding: 1rem 0;
  }

  .divider {
    width: 100%;
    height: 0.0625rem;
    background-color: var(--border-color);
  }
</style>
