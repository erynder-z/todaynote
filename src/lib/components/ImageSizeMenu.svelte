<script lang="ts">
  /**
   * Floating size/alignment/caption menu for images in the Milkdown editor.
   *
   * Listens for clicks on images within the editor container, anchors the
   * menu to the clicked image, and persists changes (size, alignment,
   * caption) directly on the image node's attributes.
   *
   * Positioning follows the approach of SelectionFloatingAnchor: the menu
   * is clamped to the viewport (flipping above the image when there is not
   * enough room below) and re-anchored on scroll/resize so it moves
   * together with the note instead of closing.
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

  /** Distance kept between the menu and the viewport edges / the image. */
  const SAFE_MARGIN = 12;
  const SPACING = 6;
  /** Fallback menu size used before the element can be measured. */
  const FALLBACK_WIDTH = 230;
  const FALLBACK_HEIGHT = 118;

  /**
   * Menu state for the image that was clicked last.
   */
  let menu = $state<{
    pos: number;
    size: ImageSize | null;
    align: ImageAlign | null;
    caption: string;
  } | null>(null);
  let menuX = $state(0);
  let menuY = $state(0);
  let menuEl: HTMLDivElement | null = $state(null);

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
   * Returns the live DOM element of the anchored image, or null when the
   * document position no longer points at an image block.
   */
  const getAnchorEl = (): HTMLElement | null => {
    if (!menu || !instance) return null;
    const view = instance.ctx.get(editorViewCtx);
    const node = view.state.doc.nodeAt(menu.pos);
    if (node?.type.name !== 'image-block') return null;

    const dom = view.nodeDOM(menu.pos) as HTMLElement | null;
    if (!dom) return null;
    return dom.tagName === 'IMG' ? dom : (dom.querySelector('img') ?? dom);
  };

  /**
   * Anchors the menu to the image: below it when there is room, above it
   * otherwise, always clamped to the viewport. Closes the menu when the
   * image is scrolled fully out of view.
   */
  const positionMenu = () => {
    if (!menu) return;
    const anchor = getAnchorEl();
    if (!anchor) {
      menu = null;
      return;
    }

    const rect = anchor.getBoundingClientRect();
    const viewportWidth = window.innerWidth;
    const viewportHeight = window.innerHeight;

    if (rect.bottom < 0 || rect.top > viewportHeight) {
      menu = null;
      return;
    }

    const menuWidth = menuEl?.offsetWidth ?? FALLBACK_WIDTH;
    const menuHeight = menuEl?.offsetHeight ?? FALLBACK_HEIGHT;

    // Keep the menu horizontally inside the viewport, left-aligned with
    // the image where possible.
    const maxX = viewportWidth - menuWidth - SAFE_MARGIN;
    menuX =
      maxX < SAFE_MARGIN
        ? Math.max(SAFE_MARGIN, (viewportWidth - menuWidth) / 2)
        : Math.max(SAFE_MARGIN, Math.min(rect.left, maxX));

    // Prefer below the image, flip above when there is not enough room.
    const belowY = rect.bottom + SPACING;
    if (belowY + menuHeight <= viewportHeight - SAFE_MARGIN) {
      menuY = belowY;
    } else {
      const aboveY = rect.top - menuHeight - SPACING;
      menuY =
        aboveY >= SAFE_MARGIN
          ? aboveY
          : Math.max(SAFE_MARGIN, viewportHeight - menuHeight - SAFE_MARGIN);
    }
  };

  /**
   * Maps the clicked image element to its image node in the document and
   * opens the menu anchored to it.
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

    menu = {
      pos,
      size: node.attrs.size,
      align: node.attrs.align,
      caption: String(node.attrs.caption ?? ''),
    };
    // Anchor immediately with the fallback size; the effect below
    // re-measures once the menu element is mounted.
    positionMenu();
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

    // Size/alignment changes resize the figure; re-anchor on the next frame.
    requestAnimationFrame(() => positionMenu());
  };

  // Re-anchor with the measured menu size once the element is mounted.
  $effect(() => {
    if (menu && menuEl) positionMenu();
  });

  // Follow the image while the note scrolls instead of closing the menu.
  $effect(() => {
    if (!menu) return;
    const reposition = () => positionMenu();
    document.addEventListener('scroll', reposition, true);
    window.addEventListener('resize', reposition);
    return () => {
      document.removeEventListener('scroll', reposition, true);
      window.removeEventListener('resize', reposition);
    };
  });
</script>

<svelte:document
  onclick={handleClick}
  onkeydown={(e) => {
    if (e.key === 'Escape') menu = null;
  }}
/>

{#if menu}
  <div
    bind:this={menuEl}
    class="image-size-menu"
    style="top: {menuY}px; left: {menuX}px"
  >
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
    background: color-mix(in srgb, var(--accent), transparent 90%);
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
