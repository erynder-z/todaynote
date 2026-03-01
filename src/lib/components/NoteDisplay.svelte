<script lang="ts">
  export let noteContent: string;
</script>

<div class="note-display">
  <pre>{noteContent}</pre>
</div>

<style>
  .note-display {
    white-space: pre-wrap;
    word-wrap: break-word;
    padding: 1.5rem;
    background-color: var(--bg-surface);
    border: 1px solid var(--border);
    color: var(--text-main);
    border-radius: 12px;
    margin: 1rem 0;
    max-height: 70vh;
    overflow-y: auto;
  }

  pre {
    margin: 0;
    font-family: 'JetBrains Mono', 'Fira Code', monospace;
    font-size: 0.95rem;
    line-height: 1.6;
  }
</style>
