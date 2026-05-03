<script lang="ts">
  /**
   * Reusable footer component for modal dialogs.
   * Displays keyboard shortcuts and optional item count.
   */
  interface Shortcut {
    label: string;
    key?: string;
  }

  let {
    shortcuts,
    count = null,
    countLabel = '',
  }: {
    shortcuts: Shortcut[];
    count?: number | null;
    countLabel?: string;
  } = $props();
</script>

<footer class="modal-footer">
  <div class="shortcuts">
    {#each shortcuts as shortcut}
      <div class="shortcut-item">
        <span>{shortcut.label}</span>
        {#if shortcut.key}
          <span class="key">{shortcut.key}</span>
        {/if}
      </div>
    {/each}
  </div>
  {#if count !== null}
    <div class="count">
      {countLabel || ''}
    </div>
  {/if}
</footer>

<style>
  .modal-footer {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.75rem;
    padding: 0.5rem 1rem;
    background-color: var(--bg-surface);
    border-top: 1px solid var(--border);
    font-size: 0.75rem;
    color: var(--text-main);
  }

  .shortcuts {
    display: flex;
    gap: 2rem;
    align-items: flex-start;
  }

  .shortcut-item {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .key {
    background-color: var(--bg-main);
    border: 1px solid var(--border);
    padding: 0.1rem 0.3rem;
    border-radius: 3px;
    color: var(--text-muted);
    font-family: var(--font-mono);
  }

  .count {
    color: var(--text-muted);
  }
</style>
