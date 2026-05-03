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

  let hasModifiers = $derived(primary || secondary);
</script>

<span class="keyboard-shortcut" class:boxed={!hasModifiers}>
  {#if primary}
    <span class="mod">{inputManager.primaryLabel}</span>
  {/if}
  {#if secondary}
    <span class="mod">{inputManager.secondaryLabel}</span>
  {/if}
  {#if key}
    <span class="key">{key}</span>
  {/if}
</span>

<style>
  .keyboard-shortcut {
    font-family: var(--font-mono, monospace);
    font-size: 0.7rem;
    color: var(--text-muted);
    display: inline-flex;
    align-items: center;
    gap: 0.1rem;
  }
  .keyboard-shortcut.boxed {
    background-color: var(--bg-main);
    border: 1px solid var(--border);
    padding: 0.1rem 0.3rem;
    border-radius: 3px;
  }
  .mod {
    opacity: 0.7;
  }
  .key {
    font-weight: 600;
  }
</style>
