<script lang="ts">
  /**
   * Reusable component for displaying keyboard shortcuts.
   * Supports primary modifier (Cmd/Ctrl), secondary modifier (Option/Shift), and a key.
   */
  import { inputManager } from '$lib/stores/input.svelte';

  let {
    primary = false,
    secondary = false,
    key = '',
  }: {
    primary?: boolean;
    secondary?: boolean;
    key?: string;
  } = $props();

  const isMultiKey = $derived(key.includes(',') && key.length > 5);
  const displayKey = $derived(
    key === ' ' ? 'Space' : isMultiKey ? '1-9, A-K' : key,
  );
</script>

<span class="keyboard-shortcut">
  {#if primary}
    <kbd class="kbd-key">{inputManager.primaryLabel}</kbd>
    <span class="separator">+</span>
  {/if}
  {#if secondary}
    <kbd class="kbd-key">{inputManager.secondaryLabel}</kbd>
    <span class="separator">+</span>
  {/if}
  {#if key}
    <kbd class="kbd-key" class:boxed={!primary && !secondary}>{displayKey}</kbd>
  {/if}
</span>

<style>
  .keyboard-shortcut {
    font-family: var(--font-mono, monospace);
    display: inline-flex;
    align-items: center;
    gap: 0.2rem;
  }

  .kbd-key {
    background-color: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: 4px;
    padding: 0.1rem 0.4rem;
    font-size: 0.7rem;
    color: var(--text-main);
    font-weight: 600;
    min-width: 1.2rem;
    text-align: center;
    box-shadow: 0 1px 0 var(--border);
  }

  .kbd-key.boxed {
    background-color: var(--bg-main);
  }

  .separator {
    font-size: 0.7rem;
    color: var(--text-muted);
    opacity: 0.6;
  }
</style>
