<script lang="ts">
  /**
   * Reusable footer component for modal dialogs.
   * Displays keyboard shortcuts and optional item count.
   */

  import type { ShortcutHint } from '$lib/interfaces/ui';
  import { settings } from '$lib/stores/settings.svelte';
  import KeyboardShortcut from './KeyboardShortcut.svelte';

  let {
    shortcuts,
    count = null,
    countLabel = '',
  }: {
    shortcuts: ShortcutHint[];
    count?: number | null;
    countLabel?: string;
  } = $props();
</script>

<footer class="modal-footer">
  <div class="shortcuts">
    {#each shortcuts as shortcut}
      <div class="shortcut-item">
        <span>{shortcut.label}</span>
        {#if shortcut.action && settings.shortcuts[shortcut.action]}
          {@const config = settings.shortcuts[shortcut.action]}
          <KeyboardShortcut
            key={config?.key}
            primary={config?.primary}
            secondary={config?.secondary}
          />
        {:else if shortcut.key}
          <KeyboardShortcut
            key={shortcut.key}
            primary={shortcut.primary}
            secondary={shortcut.secondary}
          />
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
    gap: 0.375rem;
    padding: 0.25rem 1rem;
    background-color: var(--bg-surface);
    font-size: 0.7rem;
    color: var(--text-main);
  }

  .shortcuts {
    display: flex;
    gap: 1.5rem;
    flex-wrap: wrap;
    justify-content: space-between;
  }

  .shortcut-item {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .count {
    color: var(--text-muted);
  }
</style>
