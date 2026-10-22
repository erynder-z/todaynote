<script lang="ts">
  /**
   * Floating size/alignment/caption menu for images in the Milkdown editor.
   *
   * Listens for clicks on images within the editor container, shows the
   * menu below the clicked image, and persists changes (size, alignment,
   * caption) directly on the image node's attributes.
   */

  import type { Editor } from '@milkdown/kit/core';
  import { editorViewCtx } from '@milkdown/kit/core';
  import type { ImageSizeMenuProps } from '#lib/interfaces/editor.ts';
  import { t } from '../utils/i18n.ts';
  import {
    IMAGE_ALIGNS,
    IMAGE_SIZES,
    type ImageAlign,
    type ImageSize,
  } from '../utils/images.ts';

  let { container, instance }: ImageSizeMenuProps = $props();

  /**
   * Menu state for the image that was clicked last.
   */
  let menu = $state<{
    x: number;
    y: number;
    pos: number;
    size: ImageSize | null;
    align: ImageAlign | null;
    caption: string;
  } | null>(null);

  /**
   * Opens the menu for a clicked image, or closes it when clicking
   * anywhere outside of the menu and images.
   */
  const handleClick = (e: MouseEvent) => {
    const target = e.target as HTMLElement;

    const img = target.closest('img');
    if (img && container?.contains(img) && instance) {
      openMenu(img, instance);
      return;
    }

    if (target.closest('.image-size-menu') || target.closest('.img-figure'))
      return;

    menu = null;
  };

  /**
   * Maps the clicked image element to its image node in the document and
   * positions the menu below it.
   */
  const openMenu = (img: HTMLImageElement, instance: Editor) => {
    const view = instance.ctx.get(editorViewCtx);

    let pos = view.posAtDOM(img, 0);
    let node = view.state.doc.nodeAt(pos);
    if (node?.type.name !== 'image-block') {
      const prev = view.state.doc.nodeAt(pos - 1);
      if (prev?.type.name === 'image-block') {
        pos = pos - 1;
        node = prev;
      }
    }
    if (node?.type.name !== 'image-block') return;

    const rect = img.getBoundingClientRect();
    menu = {
      x: rect.left,
      y: rect.bottom + 6,
      pos,
      size: node.attrs.size,
      align: node.attrs.align,
      caption: String(node.attrs.caption ?? ''),
    };
  };

  /**
   * Persists a size, alignment or caption change on the image node's
   * attributes.
   */
  const updateImage = (updates: {
    size?: ImageSize;
    align?: ImageAlign;
    caption?: string;
  }) => {
    if (!menu || !instance) return;

    const view = instance.ctx.get(editorViewCtx);
    const node = view.state.doc.nodeAt(menu.pos);
    if (node?.type.name !== 'image-block') return;

    const nextSize = updates.size ?? menu.size;
    const nextAlign = updates.align ?? menu.align;
    const nextCaption = updates.caption ?? menu.caption;

    view.dispatch(
      view.state.tr.setNodeMarkup(menu.pos, undefined, {
        ...node.attrs,
        size: nextSize,
        align: nextAlign,
        caption: nextCaption,
      }),
    );

    // Keep the menu open and reflect the new state in its highlights.
    menu = {
      ...menu,
      size: nextSize,
      align: nextAlign,
      caption: nextCaption,
    };
  };
</script>

<svelte:document
  onclick={handleClick}
  onkeydown={(e) => {
    if (e.key === 'Escape') menu = null;
  }}
/>
<svelte:window onscroll={() => (menu = null)} />

{#if menu}
  <div class="image-size-menu" style="top: {menu.y}px; left: {menu.x}px">
    <div class="menu-row">
      {#each IMAGE_SIZES as size (size)}
        <button
          class:selected={menu.size === size}
          onclick={() => updateImage({ size })}
        >
          {$t(`image.size_${size}`)}
        </button>
      {/each}
    </div>
    <div class="menu-row">
      {#each IMAGE_ALIGNS as align (align)}
        <button
          class:selected={(menu.align ?? 'left') === align}
          onclick={() => updateImage({ align })}
        >
          {$t(`image.align_${align}`)}
        </button>
      {/each}
    </div>
    <div class="menu-row">
      <!-- svelte-ignore a11y_autofocus -->
      <input
        type="text"
        value={menu.caption}
        placeholder={$t('image.caption_placeholder')}
        spellcheck="false"
        oninput={(e) => updateImage({ caption: e.currentTarget.value })}
      />
    </div>
  </div>
{/if}

<style>
  .image-size-menu {
    position: fixed;
    z-index: 2000;
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
    padding: 0.25rem;
    background-color: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: 0.5rem;
    box-shadow: 0 4px 16px rgb(0 0 0 / 20%);
  }

  .menu-row {
    display: flex;
    gap: 0.25rem;
  }

  .menu-row + .menu-row {
    border-top: 1px solid var(--border);
    padding-top: 0.2rem;
  }

  .menu-row input {
    flex: 1;
    width: 12rem;
    padding: 0.4rem 0.5rem;
    background: none;
    border: 1px solid var(--border);
    border-radius: 0.375rem;
    color: var(--text-main);
    font-size: 0.8rem;
    outline: none;
  }

  .menu-row input:focus {
    border-color: var(--accent);
  }

  .image-size-menu button {
    padding: 0.4rem 0.75rem;
    background: none;
    border: none;
    border-radius: 0.375rem;
    color: var(--text-muted);
    font-size: 0.8rem;
    font-weight: 600;
    cursor: pointer;
  }

  .image-size-menu button:hover {
    background-color: color-mix(in srgb, var(--accent), transparent 90%);
    color: var(--accent);
  }

  .image-size-menu button.selected {
    background-color: var(--accent);
    color: var(--accent-text);
  }
</style>
