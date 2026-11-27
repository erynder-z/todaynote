<script lang="ts">
  /**
   * A single action button for the thread options menu.
   */
  import KeyboardShortcut from './KeyboardShortcut.svelte';

  let {
    label,
    title,
    shortcutKey = '',
    active = false,
    disabled = false,
    onclick,
    children,
  }: {
    label: string;
    title: string;
    shortcutKey?: string;
    active?: boolean;
    disabled?: boolean;
    onclick?: () => void;
    children: import('svelte').Snippet;
  } = $props();
</script>

<button class="action-button" class:active {disabled} {title} {onclick}>
  {@render children()}
  <span>{label}</span>
  {#if shortcutKey}
    <div class="shortcut-hint">
      <KeyboardShortcut primary secondary key={shortcutKey} />
    </div>
  {/if}
</button>

<style>
  .action-button {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    padding: 1rem;
    background-color: color-mix(in srgb, var(--bg-base), transparent 60%);
    border: none;
    border-radius: 0.25rem;
    color: var(--text-muted);
    cursor: pointer;
    transition:
      background-color 0.15s cubic-bezier(0.2, 0, 0, 1),
      color 0.15s cubic-bezier(0.2, 0, 0, 1),
      opacity 0.15s cubic-bezier(0.2, 0, 0, 1);
    overflow: hidden;
    flex: 1 1 0;
    min-width: 0;
    position: relative;
  }

  .action-button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
    pointer-events: none;
  }

  .action-button :global(svg) {
    height: 1.75rem;
    width: 1.75rem;
  }

  .action-button span {
    font-size: 0.8rem;
    font-weight: 600;
    letter-spacing: 0.05em;
    text-align: center;
  }

  .shortcut-hint {
    display: flex;
    align-items: center;
    gap: 0.2rem;
    background-color: var(--bg-main);
    padding: 0.1rem 0.3rem;
    border-radius: 0.15rem;
    font-family: var(--font-mono);
    font-size: 0.7rem;
    color: var(--text-muted);
  }

  @media (max-width: 768px) {
    .action-button {
      padding: 0.75rem;
      min-width: 60px;
    }

    .action-button span {
      font-size: 0.8rem;
    }
  }
</style>
