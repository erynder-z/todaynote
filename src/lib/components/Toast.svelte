<script lang="ts">
  import { fly } from 'svelte/transition';
  import { toast } from '$lib/stores/toast.svelte';
  import type { Toast } from '$lib/types/ui';

  let { t } = $props<{ t: Toast }>();
</script>

<div
  class="toast {t.type}"
  transition:fly={{ y: 20, duration: 300 }}
  role="alert"
>
  <div class="icon">
    {#if t.type === 'error'}
      <svg
        xmlns="http://www.w3.org/2000/svg"
        height="1.25rem"
        viewBox="0 -960 960 960"
        width="1.25rem"
        fill="currentColor"
      >
        <path
          d="M480-280q17 0 28.5-11.5T520-320q0-17-11.5-28.5T480-360q-17 0-28.5 11.5T440-320q0 17 11.5 28.5T480-280Zm-40-160h80v-240h-80v240Zm40 360q-83 0-156-31.5T197-197q-54-54-85.5-127T80-480q0-83 31.5-156T197-763q54-54 127-85.5T480-880q83 0 156 31.5T763-763q54 54 85.5 127T880-480q0 83-31.5 156T763-197q-54 54-127 85.5T480-80Zm0-80q134 0 227-93t93-227q0-134-93-227t-227-93q-134 0-227 93t-93 227q0 134 93 227t227 93Zm0-320Z"
        />
      </svg>
    {:else if t.type === 'success'}
      <svg
        xmlns="http://www.w3.org/2000/svg"
        height="1.25rem"
        viewBox="0 -960 960 960"
        width="1.25rem"
        fill="currentColor"
      >
        <path
          d="m424-296 282-282-56-56-226 226-114-114-56 56 170 170Zm56 216q-83 0-156-31.5T197-197q-54-54-85.5-127T80-480q0-83 31.5-156T197-763q54-54 127-85.5T480-880q83 0 156 31.5T763-763q54 54 85.5 127T880-480q0 83-31.5 156T763-197q-54 54-127 85.5T480-80Zm0-80q134 0 227-93t93-227q0-134-93-227t-227-93q-134 0-227 93t-93 227q0 134 93 227t227 93Zm0-320Z"
        />
      </svg>
    {:else if t.type === 'warning'}
      <svg
        xmlns="http://www.w3.org/2000/svg"
        height="1.25rem"
        viewBox="0 -960 960 960"
        width="1.25rem"
        fill="currentColor"
      >
        <path
          d="M480-280q17 0 28.5-11.5T520-320q0-17-11.5-28.5T480-360q-17 0-28.5 11.5T440-320q0 17 11.5 28.5T480-280Zm-40-160h80v-240h-80v240Zm40 360q-83 0-156-31.5T197-197q-54-54-85.5-127T80-480q0-83 31.5-156T197-763q54-54 127-85.5T480-880q83 0 156 31.5T763-763q54 54 85.5 127T880-480q0 83-31.5 156T763-197q-54 54-127 85.5T480-80Zm0-80q134 0 227-93t93-227q0-134-93-227t-227-93q-134 0-227 93t-93 227q0 134 93 227t227 93Zm0-320Z"
        />
      </svg>
    {:else}
      <svg
        xmlns="http://www.w3.org/2000/svg"
        height="1.25rem"
        viewBox="0 -960 960 960"
        width="1.25rem"
        fill="currentColor"
      >
        <path
          d="M440-280h80v-240h-80v240Zm40-320q17 0 28.5-11.5T520-640q0-17-11.5-28.5T480-680q-17 0-28.5 11.5T440-640q0 17 11.5 28.5T480-600Zm0 520q-83 0-156-31.5T197-197q-54-54-85.5-127T80-480q0-83 31.5-156T197-763q54-54 127-85.5T480-880q83 0 156 31.5T763-763q54 54 85.5 127T880-480q0 83-31.5 156T763-197q-54 54-127 85.5T480-80Zm0-80q134 0 227-93t93-227q0-134-93-227t-227-93q-134 0-227 93t-93 227q0 134 93 227t227 93Zm0-320Z"
        />
      </svg>
    {/if}
  </div>
  <div class="message">{t.message}</div>
  <button class="close" onclick={() => toast.remove(t.id)} aria-label="Close">
    <svg
      xmlns="http://www.w3.org/2000/svg"
      height="1rem"
      viewBox="0 -960 960 960"
      width="1rem"
      fill="currentColor"
    >
      <path
        d="m256-200-56-56 224-224-224-224 56-56 224 224 224-224 56 56-224 224 224 224-56 56-224-224-224 224Z"
      />
    </svg>
  </button>
</div>

<style>
  .toast {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.75rem 1rem;
    background: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: 0.5rem;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
    min-width: 15rem;
    max-width: 25rem;
    pointer-events: auto;
    margin-bottom: 0.5rem;
  }

  .icon {
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .message {
    flex: 1;
    font-size: 0.875rem;
    color: var(--text-main);
    line-height: 1.4;
  }

  .close {
    background: none;
    border: none;
    padding: 0.25rem;
    cursor: pointer;
    color: var(--text-muted);
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 0.25rem;
    transition:
      background 0.2s,
      color 0.2s;
  }

  .close:hover {
    background: var(--bg-hover);
    color: var(--text-main);
  }

  .error {
    border-left: 4px solid var(--error);
  }
  .error .icon {
    color: var(--error);
  }

  .success {
    border-left: 4px solid var(--success);
  }
  .success .icon {
    color: var(--success);
  }

  .warning {
    border-left: 4px solid var(--warning);
  }
  .warning .icon {
    color: var(--warning);
  }

  .info {
    border-left: 4px solid var(--accent);
  }
  .info .icon {
    color: var(--accent);
  }
</style>
