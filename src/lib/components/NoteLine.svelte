<script lang="ts">
  import { tick } from 'svelte';
  import type { NoteLineProps } from '$lib/types/notes';
  import { renderMarkdown } from '$lib/utils/notes';

  let {
    markdown = $bindable(),
    isActive,
    onActivate,
    onDeactivate,
    onChange,
    onKeyDown,
  }: NoteLineProps = $props();

  let html = $state('');
  let textarea: HTMLTextAreaElement | null = $state(null);

  const updateHtml = async () => {
    html = await renderMarkdown(markdown || '&nbsp;');
    onChange(markdown, html);
  };

  const autoResize = (node: HTMLTextAreaElement) => {
    const resize = () => {
      node.style.height = 'auto';
      node.style.height = `${node.scrollHeight}px`;
    };
    node.addEventListener('input', resize);
    tick().then(resize);
    return {
      destroy: () => node.removeEventListener('input', resize),
    };
  };

  const focusAndSelect = () => {
    if (isActive && textarea) {
      textarea.focus();
      const length = textarea.value.length;
      textarea.setSelectionRange(length, length);
    }
  };

  $effect(() => {
    if (markdown !== undefined) updateHtml();
  });

  $effect(() => focusAndSelect());
</script>

<div class="line-wrapper">
  {#if isActive}
    <textarea
      bind:this={textarea}
      value={markdown}
      oninput={(e) => {
        markdown = e.currentTarget.value;
        updateHtml();
      }}
      onkeydown={onKeyDown}
      onblur={onDeactivate}
      use:autoResize
      spellcheck="false"
    ></textarea>
  {:else}
    <div
      role="button"
      tabindex="0"
      class="rendered-line"
      onmousedown={(e) => {
        e.preventDefault();
        onActivate();
      }}
      onkeydown={(e) => e.key === 'Enter' && onActivate()}
    >
      {@html html}
    </div>
  {/if}
</div>

<style>
  .line-wrapper {
    min-height: 1.6rem;
    margin-bottom: 0.2rem;
  }

  textarea {
    width: 100%;
    background: transparent;
    border: none;
    color: inherit;
    font-family: inherit;
    font-size: 1rem;
    line-height: 1.6;
    resize: none;
    padding: 0;
    margin: 0;
    outline: none;
    overflow: hidden;
    display: block;
  }

  .rendered-line {
    cursor: text;
    font-size: 1rem;
    line-height: 1.6;
    min-height: 1.6rem;
  }
</style>
