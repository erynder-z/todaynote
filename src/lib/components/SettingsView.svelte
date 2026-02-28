<script lang="ts">
  import FolderSelector from './FolderSelector.svelte';
  import LanguageSelector from './LanguageSelector.svelte';
</script>

<div class="settings-view">
  <FolderSelector />
  <div class="divider"></div>
  <LanguageSelector />
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
    height: 1px;
    background-color: rgba(0, 0, 0, 0.1);
  }

  @media (prefers-color-scheme: dark) {
    .divider {
      background-color: rgba(255, 255, 255, 0.1);
    }
  }
</style>
