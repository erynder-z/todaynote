<script lang="ts">
  /**
   * A pure wrapper around the Milkdown editor.
   * Handles initialization, destruction, and basic content syncing.
   */
  import {
    defaultValueCtx,
    Editor,
    editorViewCtx,
    editorViewOptionsCtx,
    rootCtx,
  } from '@milkdown/kit/core';
  import type { MilkdownPlugin } from '@milkdown/kit/ctx';
  import { history } from '@milkdown/kit/plugin/history';
  import { listener, listenerCtx } from '@milkdown/kit/plugin/listener';
  import { commonmark } from '@milkdown/kit/preset/commonmark';
  import { gfm } from '@milkdown/kit/preset/gfm';
  import { highlight, highlightPluginConfig } from '@milkdown/plugin-highlight';
  import { untrack } from 'svelte';
  import {
    activeThreadPlugin,
    setActiveThread,
  } from '../plugins/activeThreadPlugin';
  import { threadMarkerPlugin } from '../plugins/threadMarkerPlugin';
  import { parser } from '../utils/milkdownConfig';

  interface Props {
    content: string;
    onUpdate?: (markdown: string) => void;
    plugins?: MilkdownPlugin[];
    readonly?: boolean;
    activeThreadIndex?: number;
    onReady?: (instance: Editor) => void;
  }

  let {
    content,
    onUpdate,
    plugins = [],
    readonly = false,
    activeThreadIndex = -1,
    onReady,
  }: Props = $props();

  const allPlugins = $derived([
    commonmark,
    gfm,
    listener,
    threadMarkerPlugin,
    activeThreadPlugin,
    highlight,
    history,
    ...plugins,
  ]);

  let container: HTMLDivElement | null = $state(null);
  let currentInstance: Editor | null = $state(null);

  $effect(() => {
    if (!container) return;

    let isDestroyed = false;

    const editor = Editor.make().config((ctx) => {
      ctx.set(rootCtx, container);
      ctx.set(
        defaultValueCtx,
        untrack(() => content),
      );
      ctx.set(editorViewOptionsCtx, {
        editable: () => !readonly,
      });

      // Use pre-created parser for code syntax highlighting
      ctx.set(highlightPluginConfig.key, { parser });

      ctx.get(listenerCtx).markdownUpdated((_, markdown) => {
        if (onUpdate) onUpdate(markdown);
      });
    });

    // Use all plugins from the memoized array
    for (const plugin of allPlugins) editor.use(plugin);

    editor.create().then((inst) => {
      if (isDestroyed) {
        inst.destroy();
      } else {
        currentInstance = inst;
        // Force layout recalculation to prevent paint suppression
        requestAnimationFrame(() => {
          if (container && !isDestroyed) {
            void container.offsetHeight;
            onReady?.(inst);
          }
        });
      }
    });

    return () => {
      isDestroyed = true;
      currentInstance?.destroy();
      currentInstance = null;
    };
  });

  // Update readonly state without recreating the editor
  $effect(() => {
    if (!currentInstance?.action) return;

    currentInstance.action((ctx) => {
      const options = ctx.get(editorViewOptionsCtx);
      options.editable = () => !readonly;
    });
  });

  $effect(() => {
    if (!currentInstance?.action) return;

    const view = currentInstance.ctx.get(editorViewCtx);

    setActiveThread(view, activeThreadIndex);
  });
</script>

<div
  bind:this={container}
  class="milkdown-editor-wrapper"
  data-active-thread={activeThreadIndex !== -1 ? activeThreadIndex : undefined}
></div>

<style>
  .milkdown-editor-wrapper {
    width: 100%;
    height: 100%;
    min-height: 70dvh;
  }

  .milkdown-editor-wrapper :global(.milkdown) {
    width: 100%;
    height: 100%;
    background: transparent;
    color: var(--text-main);
    font-family: inherit;
    font-size: 1rem;
    line-height: 1.6;
  }

  .milkdown-editor-wrapper :global(.milkdown .editor) {
    outline: none;
    padding-bottom: 5rem;
    min-height: 100%;
  }

  /* Thread Markers */
  .milkdown-editor-wrapper :global(.milkdown .thread-marker) {
    color: var(--thread-marker-color);
    font-size: 1.825rem;
    cursor: default;
    user-select: none;
    border-bottom: 0.2rem solid
      color-mix(in srgb, var(--md-h1), transparent 60%);
    padding-bottom: 0.5rem;
  }

  .milkdown-editor-wrapper :global(.milkdown .thread-marker:first-child) {
    margin-top: 0;
  }

  /* Headings */
  .milkdown-editor-wrapper :global(.milkdown .thread-marker),
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
    font-size: 1.5rem;
    border-bottom: 0.085rem solid
      color-mix(in srgb, var(--md-h2), transparent 80%);
    padding-bottom: 0.3rem;
  }

  .milkdown-editor-wrapper :global(.milkdown h2) {
    color: var(--md-h2);
    font-size: 1.25rem;
  }
  .milkdown-editor-wrapper :global(.milkdown h3) {
    color: var(--md-h3);
    font-size: 1.15rem;
  }
  .milkdown-editor-wrapper :global(.milkdown h4) {
    color: var(--md-h4);
    font-size: 1.1rem;
  }

  /* Links */
  .milkdown-editor-wrapper :global(.milkdown a) {
    color: var(--md-link);
    text-decoration: underline;
    text-underline-offset: 2px;
    transition: color 0.15s cubic-bezier(0.2, 0, 0, 1);
    cursor: pointer;
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

  /* Code Syntax Highlighting */
  .milkdown-editor-wrapper :global(.milkdown pre code .hljs-comment),
  .milkdown-editor-wrapper :global(.milkdown pre code .hljs-quote) {
    color: var(--text-ui-muted);
    font-style: italic;
  }

  .milkdown-editor-wrapper :global(.milkdown pre code .hljs-keyword),
  .milkdown-editor-wrapper :global(.milkdown pre code .hljs-selector-tag),
  .milkdown-editor-wrapper
    :global(.milkdown pre code .hljs-meta .hljs-keyword) {
    color: var(--md-bold);
    font-weight: 600;
  }

  .milkdown-editor-wrapper :global(.milkdown pre code .hljs-string),
  .milkdown-editor-wrapper :global(.milkdown pre code .hljs-regexp),
  .milkdown-editor-wrapper :global(.milkdown pre code .hljs-addition),
  .milkdown-editor-wrapper :global(.milkdown pre code .hljs-meta .hljs-string) {
    color: var(--success);
  }

  .milkdown-editor-wrapper :global(.milkdown pre code .hljs-number),
  .milkdown-editor-wrapper :global(.milkdown pre code .hljs-literal) {
    color: var(--md-h3);
  }

  .milkdown-editor-wrapper :global(.milkdown pre code .hljs-title),
  .milkdown-editor-wrapper :global(.milkdown pre code .hljs-title.function_),
  .milkdown-editor-wrapper :global(.milkdown pre code .hljs-section),
  .milkdown-editor-wrapper :global(.milkdown pre code .hljs-name) {
    color: var(--md-h1);
  }

  .milkdown-editor-wrapper :global(.milkdown pre code .hljs-title.class_),
  .milkdown-editor-wrapper :global(.milkdown pre code .hljs-class .hljs-title),
  .milkdown-editor-wrapper :global(.milkdown pre code .hljs-type),
  .milkdown-editor-wrapper :global(.milkdown pre code .hljs-built_in) {
    color: var(--md-italic);
  }

  .milkdown-editor-wrapper :global(.milkdown pre code .hljs-attr),
  .milkdown-editor-wrapper :global(.milkdown pre code .hljs-attribute),
  .milkdown-editor-wrapper :global(.milkdown pre code .hljs-variable),
  .milkdown-editor-wrapper :global(.milkdown pre code .hljs-template-variable) {
    color: var(--md-h2);
  }

  .milkdown-editor-wrapper :global(.milkdown pre code .hljs-operator),
  .milkdown-editor-wrapper :global(.milkdown pre code .hljs-punctuation) {
    color: var(--text-ui-muted);
  }

  .milkdown-editor-wrapper :global(.milkdown pre code .hljs-params) {
    color: var(--md-code-text);
  }

  .milkdown-editor-wrapper :global(.milkdown pre code .hljs-doctag) {
    color: var(--md-quote-border);
  }

  .milkdown-editor-wrapper :global(.milkdown pre code .hljs-meta) {
    color: var(--md-h4);
  }

  .milkdown-editor-wrapper :global(.milkdown pre code .hljs-deletion) {
    color: var(--error);
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

  /* Thread options active */
  .milkdown-editor-wrapper :global(.thread-marker[data-active-thread='true']) {
    background-color: color-mix(in srgb, var(--accent), transparent 92%);
    border-bottom-color: var(--accent);
    border-radius: 0;
    padding-left: 0.5rem;
    transition:
      background-color 150ms ease,
      border-color 150ms ease;
  }
</style>
