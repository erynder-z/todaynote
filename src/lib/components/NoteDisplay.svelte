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
    padding: 2rem;
    background-color: var(--bg-surface);
    border: 0.0625rem solid var(--border);
    color: var(--text-main);
    border-radius: 0.75rem;
    margin: 1rem 0;
    max-height: 70vh;
    overflow-y: auto;
    tab-size: 4;
    -moz-tab-size: 4;
  }

  pre {
    margin: 0;
    font-family: inherit;
    font-size: 1rem;
    line-height: 1.6;
    font-weight: 400;
  }
</style>
