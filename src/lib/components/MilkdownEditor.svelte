<script lang="ts">
  /**
   * A pure wrapper around the Milkdown editor.
   * Handles initialization, destruction, and basic content syncing.
   */
  import { defaultValueCtx, Editor, rootCtx } from '@milkdown/core';
  import type { MilkdownPlugin } from '@milkdown/ctx';
  import { listener, listenerCtx } from '@milkdown/plugin-listener';
  import { commonmark } from '@milkdown/preset-commonmark';
  import { gfm } from '@milkdown/preset-gfm';
  import { untrack } from 'svelte';

  let {
    content,
    onUpdate,
    instance = $bindable(null),
    plugins = [],
  } = $props<{
    content: string;
    onUpdate?: (markdown: string) => void;
    instance?: Editor | null;
    plugins?: MilkdownPlugin[];
  }>();

  let container: HTMLDivElement | null = $state(null);

  $effect(() => {
    if (!container) return;

    let isDestroyed = false;
    const editor = Editor.make()
      .config((ctx) => {
        ctx.set(rootCtx, container);
        ctx.set(
          defaultValueCtx,
          untrack(() => content),
        );
        ctx.get(listenerCtx).markdownUpdated((_, markdown) => {
          if (onUpdate) onUpdate(markdown);
        });
      })
      .use(commonmark)
      .use(gfm)
      .use(listener);

    // Add any plugins provided via props
    for (const plugin of plugins) editor.use(plugin);

    editor.create().then((inst) => {
      if (isDestroyed) {
        inst.destroy();
      } else {
        instance = inst;
      }
    });

    return () => {
      isDestroyed = true;
      instance?.destroy();
      instance = null;
    };
  });
</script>

<div bind:this={container} class="milkdown-editor-wrapper"></div>

<style>
  .milkdown-editor-wrapper :global(.milkdown) {
    width: 100%;
    min-height: 70dvh;
    background: transparent;
    color: var(--text-main);
    font-family: inherit;
    font-size: 1rem;
    line-height: 1.6;
  }

  .milkdown-editor-wrapper :global(.milkdown .editor) {
    outline: none;
    padding-bottom: 5rem;
  }

  /* Headings */
  .milkdown-editor-wrapper :global(.milkdown h1),
  .milkdown-editor-wrapper :global(.milkdown h2),
  .milkdown-editor-wrapper :global(.milkdown h3),
  .milkdown-editor-wrapper :global(.milkdown h4) {
    font-weight: 600;
    margin-top: 2rem;
    margin-bottom: 1rem;
    line-height: 1.3;
  }

  .milkdown-editor-wrapper :global(.milkdown h1) {
    color: var(--md-h1);
    font-size: 1.875rem;
    border-bottom: 0.2rem solid
      color-mix(in srgb, var(--md-h1), transparent 60%);
    padding-bottom: 0.5rem;
  }

  .milkdown-editor-wrapper :global(.milkdown h1):first-child {
    margin-top: 0;
  }

  .milkdown-editor-wrapper :global(.milkdown h2) {
    color: var(--md-h2);
    font-size: 1.5rem;
    border-bottom: 0.085rem solid
      color-mix(in srgb, var(--md-h2), transparent 80%);
    padding-bottom: 0.3rem;
  }
  .milkdown-editor-wrapper :global(.milkdown h3) {
    color: var(--md-h3);
    font-size: 1.25rem;
  }
  .milkdown-editor-wrapper :global(.milkdown h4) {
    color: var(--md-h4);
    font-size: 1.125rem;
  }

  /* Links */
  .milkdown-editor-wrapper :global(.milkdown a) {
    color: var(--md-link);
    text-decoration: underline;
    text-underline-offset: 2px;
    transition: color 0.15s cubic-bezier(0.2, 0, 0, 1);
  }

  .milkdown-editor-wrapper :global(.milkdown a:hover) {
    color: var(--md-link-hover);
  }

  /* Lists */
  .milkdown-editor-wrapper :global(.milkdown ul),
  .milkdown-editor-wrapper :global(.milkdown ol) {
    padding-left: 2rem;
    margin: 1rem 0;
  }

  .milkdown-editor-wrapper :global(.milkdown li) {
    margin: 0.5rem 0;
  }

  .milkdown-editor-wrapper :global(.milkdown li::marker) {
    color: var(--md-list-marker);
    font-weight: 600;
  }

  .milkdown-editor-wrapper :global(.milkdown ul) {
    list-style-type: disc;
  }

  .milkdown-editor-wrapper :global(.milkdown ol) {
    list-style-type: decimal;
  }

  /* Code */
  .milkdown-editor-wrapper :global(.milkdown pre) {
    background: var(--md-code-bg);
    border: 1px solid var(--border);
    border-radius: 0.5rem;
    padding: 1rem;
    overflow-x: auto;
    margin: 1.5rem 0;
    font-family: var(--font-mono);
    font-size: 0.875rem;
    line-height: 1.5;
  }

  .milkdown-editor-wrapper :global(.milkdown pre code) {
    background: transparent;
    color: var(--md-code-text);
    padding: 0;
    border-radius: 0;
    font-size: inherit;
  }

  .milkdown-editor-wrapper :global(.milkdown code) {
    background: var(--md-code-inline-bg);
    color: var(--md-code-text);
    padding: 0.2rem 0.4rem;
    border-radius: 0.25rem;
    font-family: var(--font-mono);
    font-size: 0.875rem;
  }

  /* Blockquotes */
  .milkdown-editor-wrapper :global(.milkdown blockquote) {
    border-left: 4px solid var(--md-quote-border);
    padding-left: 1.5rem;
    margin: 1.5rem 0;
    color: var(--text-muted);
    font-style: italic;
  }

  /* Horizontal Rule */
  .milkdown-editor-wrapper :global(.milkdown hr) {
    border: 0;
    border-top: 1px solid var(--md-hr);
    margin: 2.5rem 0;
  }

  /* Spacing */
  .milkdown-editor-wrapper :global(.milkdown p) {
    margin: 1rem 0;
  }

  /* Strong/Emphasis */
  .milkdown-editor-wrapper :global(.milkdown strong) {
    font-weight: 600;
    color: var(--md-bold);
  }

  .milkdown-editor-wrapper :global(.milkdown em) {
    font-style: italic;
    color: var(--md-italic);
  }

  /* Text Selection */
  .milkdown-editor-wrapper :global(.milkdown ::selection) {
    background: var(--text-selection);
    color: var(--text-main);
  }
</style>
