<script lang="ts">
  /**
   * Reusable component for displaying keyboard shortcuts.
   * Supports primary modifier (Cmd/Ctrl), secondary modifier (Option/Shift), and a key.
   */
  import { inputManager } from '#lib/stores/input.svelte.ts';

  let {
    primary = false,
    secondary = false,
    key = '',
  }: {
    primary?: boolean;
    secondary?: boolean;
    key?: string;
  } = $props();

  const isMultiKey = $derived(key.split(',').length > 5);

  // Compact symbols for non-letter keys shown in shortcut hints.
  const KEY_LABELS: Record<string, string> = {
    ArrowDown: '↓',
    ArrowUp: '↑',
    ArrowLeft: '←',
    ArrowRight: '→',
    Backspace: '⌫',
    Escape: 'Esc',
  };

  const displayKey = $derived(
    key === ' '
      ? 'Space'
      : isMultiKey
        ? '1-9, A-K'
        : (KEY_LABELS[key] ?? key),
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
    display: flex;
    align-items: center;
    gap: 0.2rem;
  }

  .kbd-key {
    font-size: 0.8rem;
    color: var(--text-muted);
    font-weight: 600;
    min-width: 0.6rem;
    text-align: center;
  }

  .separator {
    font-size: 0.7rem;
    color: var(--text-muted);
    opacity: 0.6;
  }
</style>
