<script lang="ts">
  interface Props {
    value: string;
    placeholder?: string;
    type?: 'text' | 'password';
    oninput?: (value: string) => void;
    onkeydown?: (event: KeyboardEvent) => void;
  }

  let {
    value = $bindable(),
    placeholder = '',
    type = 'text',
    oninput,
    onkeydown
  }: Props = $props();

  function handleInput(event: Event) {
    const target = event.target as HTMLInputElement;
    value = target.value;
    oninput?.(value);
  }
</script>

<input
  {type}
  class="text-input"
  {placeholder}
  {value}
  oninput={handleInput}
  {onkeydown}
/>

<style>
  .text-input {
    flex: 1;
    background: var(--color-bg);
    border: var(--border);
    color: var(--color-primary);
    padding: var(--spacing-sm) var(--spacing-md);
    font-size: var(--font-size-base);
    font-family: var(--font-family);
  }

  .text-input::placeholder {
    color: var(--color-primary);
    opacity: 0.5;
  }

  .text-input:focus {
    outline: none;
    box-shadow: var(--glow-shadow);
  }

  @media (max-width: 768px) {
    .text-input {
      width: 100%;
    }
  }
</style>
