<script lang="ts">
  interface Props {
    value: string;
    options: string[];
    onchange?: (value: string) => void;
  }

  let {
    value = $bindable(),
    options,
    onchange
  }: Props = $props();

  let open = $state(false);

  function select(face: string) {
    value = face;
    open = false;
    onchange?.(face);
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      open = false;
    }
  }
</script>

<div class="face-select" class:open>
  <button
    type="button"
    class="face-select-trigger"
    onclick={() => open = !open}
    onkeydown={handleKeydown}
    title={value}
  >
    <img
      class="face-img"
      src="/api/faces/{value}.png"
      alt={value}
    />
    <span class="chevron"></span>
  </button>

  {#if open}
    <div class="face-select-dropdown">
      {#each options as face}
        <button
          type="button"
          class="face-option"
          class:selected={face === value}
          onclick={() => select(face)}
          title={face}
        >
          <img
            class="face-img"
            src="/api/faces/{face}.png"
            alt={face}
          />
        </button>
      {/each}
    </div>
  {/if}
</div>

{#if open}
  <button type="button" class="backdrop" aria-label="Close dropdown" onclick={() => open = false}></button>
{/if}

<style>
  .face-select {
    position: relative;
    display: inline-block;
  }

  .face-select-trigger {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
    background: var(--color-bg);
    border: var(--border);
    color: var(--color-primary);
    padding: var(--spacing-xs) var(--spacing-sm);
    font-size: var(--font-size-sm);
    font-family: var(--font-family);
    cursor: pointer;
  }

  .face-select-trigger:focus {
    outline: none;
    box-shadow: var(--glow-shadow);
  }

  .face-select.open .face-select-trigger {
    box-shadow: var(--glow-shadow);
  }

  .face-img {
    width: 40px;
    height: 30px;
    flex-shrink: 0;
    filter: sepia(1) saturate(5) hue-rotate(120deg);
  }

  .chevron::after {
    content: '\25BC';
    font-size: 10px;
    opacity: 0.7;
  }

  .face-select.open .chevron::after {
    content: '\25B2';
  }

  .face-select-dropdown {
    position: absolute;
    top: 100%;
    left: 0;
    z-index: 100;
    background: var(--color-bg);
    border: var(--border);
    box-shadow: var(--glow-shadow);
    max-height: 200px;
    overflow-y: auto;
  }

  .face-option {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
    width: 100%;
    background: transparent;
    border: none;
    color: var(--color-primary);
    padding: var(--spacing-xs) var(--spacing-sm);
    font-size: var(--font-size-sm);
    font-family: var(--font-family);
    cursor: pointer;
    text-align: left;
  }

  .face-option:hover {
    background: var(--color-bg-subtle);
  }

  .face-option.selected {
    background: var(--color-primary);
    color: var(--color-bg);
  }

  .backdrop {
    position: fixed;
    inset: 0;
    background: transparent;
    border: none;
    cursor: default;
    z-index: 99;
  }
</style>
