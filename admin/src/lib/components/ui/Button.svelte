<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    variant?: 'default' | 'small' | 'block' | 'icon';
    active?: boolean;
    disabled?: boolean;
    onclick?: (event: MouseEvent) => void;
    children: Snippet;
  }

  let { variant = 'default', active = false, disabled = false, onclick, children }: Props = $props();
</script>

<button
  class="btn"
  class:active
  class:small={variant === 'small'}
  class:block={variant === 'block'}
  class:icon={variant === 'icon'}
  {disabled}
  {onclick}
>
  {@render children()}
</button>

<style>
  .btn {
    background: var(--color-bg);
    border: var(--border);
    color: var(--color-primary);
    padding: var(--spacing-sm) var(--spacing-lg);
    cursor: pointer;
    font-size: var(--font-size-base);
    font-family: var(--font-family);
    text-transform: uppercase;
  }

  .btn:hover:not(:disabled) {
    background: var(--color-primary);
    color: var(--color-bg);
    box-shadow: var(--glow-shadow);
  }

  .btn.active {
    background: var(--color-primary);
    color: var(--color-bg);
  }

  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn.small {
    padding: var(--spacing-xs) var(--spacing-sm);
    font-size: var(--font-size-small);
  }

  .btn.block {
    width: 100%;
    text-align: left;
    text-transform: none;
    padding: var(--spacing-sm) var(--spacing-md);
  }

  .btn.icon {
    border: none;
    background: transparent;
    padding: var(--spacing-xs) var(--spacing-sm);
    opacity: 0.4;
    font-size: var(--font-size-small);
  }

  .btn.icon:hover:not(:disabled) {
    background: transparent;
    color: var(--color-primary);
    opacity: 0.8;
    box-shadow: none;
  }

  .btn.icon.active {
    background: transparent;
    color: var(--color-primary);
    opacity: 1;
    text-shadow: var(--text-glow);
  }

  @media (max-width: 768px) {
    .btn {
      padding: var(--spacing-md) var(--spacing-lg);
    }

    .btn.block {
      padding: var(--spacing-md);
      font-size: 0.8rem;
    }
  }
</style>
