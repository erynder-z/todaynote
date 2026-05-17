<script lang="ts">
  /**
   * Layout toggle toolbar for switching between list and masonry views.
   */
  import { settings } from '$lib';

  interface Props {
    onLayoutChange: (layout: 'list' | 'masonry') => void;
  }

  let { onLayoutChange }: Props = $props();
</script>

<div class="layout-toolbar">
  <div class="toggle-group">
    <button
      class="toggle-btn"
      class:active={settings.notesListLayout === 'list'}
      onclick={() => onLayoutChange('list')}
      title="List View"
    >
      <svg
        xmlns="http://www.w3.org/2000/svg"
        height="1.25rem"
        viewBox="0 -960 960 960"
        width="1.25rem"
        fill="currentColor"
        ><path
          d="M240-240v-120h480v120H240Zm0-200v-120h480v120H240Zm0-200v-120h480v120H240Z"
        /></svg
      >
    </button>
    <button
      class="toggle-btn"
      class:active={settings.notesListLayout === 'masonry'}
      onclick={() => onLayoutChange('masonry')}
      title="Masonry View"
    >
      <svg
        xmlns="http://www.w3.org/2000/svg"
        height="1.25rem"
        viewBox="0 -960 960 960"
        width="1.25rem"
        fill="currentColor"
        ><path
          d="M200-120q-33 0-56.5-23.5T120-200v-560q0-33 23.5-56.5T200-840h560q33 0 56.5 23.5T840-760v560q0-33-23.5-56.5T760-120H200Zm0-80h240v-180H200v180Zm320 0h240v-300H520v300ZM200-460h240v-300H200v300Zm320 0h240v-180H520v180Z"
        /></svg
      >
    </button>
  </div>
</div>

<style>
  .layout-toolbar {
    display: flex;
    justify-content: flex-end;
    padding: 0.75rem 1.5rem;
    border-bottom: 1px solid var(--border);
    background-color: var(--bg-surface);
  }

  .toggle-group {
    display: flex;
    background-color: var(--bg-base);
    border: 1px solid var(--border);
    border-radius: 0.5rem;
    padding: 2px;
  }

  .toggle-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0.4rem;
    background: none;
    border: none;
    border-radius: 0.35rem;
    color: var(--text-muted);
    cursor: pointer;
    transition:
      background-color 0.15s cubic-bezier(0.2, 0, 0, 1),
      color 0.15s cubic-bezier(0.2, 0, 0, 1);
  }

  .toggle-btn:hover {
    color: var(--text-main);
  }

  .toggle-btn.active {
    background-color: var(--accent);
    color: white;
  }
</style>
