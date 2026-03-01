<script lang="ts">
  import FolderSelector from './FolderSelector.svelte';
  import LanguageSelector from './LanguageSelector.svelte';
  import ThemeSelector from './ThemeSelector.svelte';
</script>

<div class="settings-view">
  <FolderSelector />
  <div class="divider"></div>
  <LanguageSelector />
  <div class="divider"></div>
  <ThemeSelector />
</div>

<style>
  .settings-view {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2rem;
    width: 100%;
    padding: 1rem 0;
  }

  .divider {
    width: 100%;
    height: 0.0625rem;
    background-color: var(--border-color);
  }
</style>
