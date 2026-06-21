<script lang="ts">
  /**
   * Sidebar component for search mode selection and navigation.
   */
  import { t } from '$lib';

  let { searchMode, selectedTag, onModeChange } = $props<{
    searchMode: 'notes' | 'threads' | 'tags';
    selectedTag: string | null;
    onModeChange: (mode: 'notes' | 'threads' | 'tags') => void;
  }>();
</script>

<aside class="sidebar">
  <div class="sidebar-header">
    <h2>{$t('search.title')}</h2>
  </div>

  <nav class="mode-nav">
    <button
      class="mode-btn"
      class:active={searchMode === 'notes' && !selectedTag}
      onclick={() => onModeChange('notes')}
    >
      <svg
        xmlns="http://www.w3.org/2000/svg"
        height="1.2rem"
        viewBox="0 -960 960 960"
        width="1.2rem"
        fill="currentColor"
        ><path
          d="M280-160v-441q0-33 24-56t57-23h439q33 0 56.5 23.5T880-600v320L680-80H360q-33 0-56.5-23.5T280-160ZM81-710q-6-33 13-59.5t52-32.5l434-77q33-6 59.5 13t32.5 52l10 54h-82l-7-40-433 77 40 226v279q-16-9-27.5-24T158-276L81-710Zm279 110v440h280v-160h160v-280H360Zm220 220Z"
        /></svg
      >
      <span>{$t('search.notes')}</span>
    </button>

    <button
      class="mode-btn"
      class:active={searchMode === 'threads'}
      onclick={() => onModeChange('threads')}
    >
      <svg
        xmlns="http://www.w3.org/2000/svg"
        height="1.2rem"
        viewBox="0 -960 960 960"
        width="1.2rem"
        fill="currentColor"
        ><path
          d="M600-80v-100L320-320H120v-240h172l108-124v-196h240v240H468L360-516v126l240 120v-50h240v240H600ZM480-720h80v-80h-80v80ZM200-400h80v-80h-80v80Zm480 240h80v-80h-80v80ZM520-760ZM240-440ZM480 240Z"
        /></svg
      >
      <span>{$t('search.threads')}</span>
    </button>

    <button
      class="mode-btn"
      class:active={searchMode === 'tags' || selectedTag}
      onclick={() => onModeChange('tags')}
    >
      <svg
        xmlns="http://www.w3.org/2000/svg"
        height="1.2rem"
        viewBox="0 -960 960 960"
        width="1.2rem"
        fill="currentColor"
        ><path
          d="m240-160 40-160H120l20-80h160l40-160H180l20-80h160l40-160h80l-40 160h160l40-160h80l-40 160h160l-20 80H660l-40 160h160l-20 80H600l-40 160h-80l40-160H360l-40 160h-80Zm140-240h160l40-160H420l-40 160Z"
        /></svg
      >
      <span>{$t('search.tags')}</span>
    </button>
  </nav>

  <div class="sidebar-spacer"></div>
</aside>

<style>
  .sidebar {
    background-color: var(--bg-main);
    border-right: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    padding: 1.25rem 0.75rem;
    gap: 1.25rem;
  }

  .sidebar-header h2 {
    font-size: 0.85rem;
    font-weight: 800;
    margin: 0 0.5rem;
    color: var(--text-muted);
    letter-spacing: 0.05em;
    text-transform: uppercase;
  }

  .mode-nav {
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
  }

  .mode-btn {
    display: flex;
    align-items: center;
    gap: 0.65rem;
    padding: 0.6rem 0.75rem;
    background: none;
    border: none;
    border-radius: 0.4rem;
    color: var(--text-muted);
    font-weight: 600;
    font-size: 0.9rem;
    cursor: pointer;
    text-align: left;
    transition: all 0.15s cubic-bezier(0.2, 0, 0, 1);
  }

  .mode-btn.active {
    background-color: color-mix(in srgb, var(--accent), transparent 90%);
    color: var(--accent);
  }

  .sidebar-spacer {
    flex: 1;
  }

  @media (max-width: 768px) {
    .sidebar {
      flex-direction: row !important;
      padding: 0.5rem !important;
      border-right: none !important;
      border-bottom: 1px solid var(--border);
      height: auto !important;
      gap: 0.5rem !important;
      align-items: center;
    }

    .sidebar-header,
    .sidebar-spacer {
      display: none !important;
    }

    .mode-nav {
      flex-direction: row !important;
      flex: 1;
    }

    .mode-btn {
      padding: 0.5rem 1rem !important;
      justify-content: center !important;
    }

    .mode-btn span {
      display: none;
    }
  }
</style>
