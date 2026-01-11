<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    title: string;
    collapsed?: boolean;
    children: Snippet;
  }

  let { title, collapsed = $bindable(false), children }: Props = $props();
</script>

<div class="section" class:collapsed>
  <h2 class="section-header">
    <button type="button" class="section-toggle" onclick={() => collapsed = !collapsed}>
      {title} <span class="collapse-icon">{collapsed ? '+' : '-'}</span>
    </button>
  </h2>
  <div class="section-content" class:collapsed>
    {@render children()}
  </div>
</div>

<style>
  .section {
    margin-bottom: var(--spacing-2xl);
  }

  .section.collapsed {
    margin-bottom: 0;
  }

  .section-header {
    margin-bottom: var(--spacing-md);
  }

  .section-toggle {
    width: 100%;
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--spacing-sm) 0;
    border: none;
    background: transparent;
    font-family: var(--font-family);
    font-size: var(--font-size-base);
    color: var(--color-primary);
    text-transform: uppercase;
    letter-spacing: 0.1em;
    opacity: 0.7;
    cursor: pointer;
    user-select: none;
  }

  .section-toggle:hover {
    opacity: 1;
  }

  .collapse-icon {
    font-size: var(--font-size-large);
    font-weight: normal;
    opacity: 0.5;
  }

  .section-content {
    overflow: hidden;
    transition: max-height 0.3s ease, opacity 0.3s ease;
    max-height: 1000px;
    opacity: 1;
  }

  .section-content.collapsed {
    max-height: 0;
    opacity: 0;
    margin-bottom: 0;
  }
</style>
