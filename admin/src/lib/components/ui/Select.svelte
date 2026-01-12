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

  function handleChange(event: Event) {
    const target = event.target as HTMLSelectElement;
    value = target.value;
    onchange?.(value);
  }
</script>

<select
  class="select"
  {value}
  onchange={handleChange}
>
  {#each options as option}
    <option value={option}>{option}</option>
  {/each}
</select>

<style>
  .select {
    background: var(--color-bg);
    border: var(--border);
    color: var(--color-primary);
    padding: var(--spacing-sm) var(--spacing-md);
    font-size: var(--font-size-base);
    font-family: var(--font-family);
    cursor: pointer;
  }

  .select:focus {
    outline: none;
    box-shadow: var(--glow-shadow);
  }

  .select option {
    background: var(--color-bg);
    color: var(--color-primary);
  }
</style>
