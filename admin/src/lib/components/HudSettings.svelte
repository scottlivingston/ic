<script lang="ts">
  import { hudState } from '../stores';
  import { toggleIpHud } from '../api';
  import Section from './ui/Section.svelte';
  import Button from './ui/Button.svelte';

  let collapsed = $state(false);

  async function handleToggleIp() {
    const newValue = !$hudState.show_ip;
    const success = await toggleIpHud(newValue);
    if (success) {
      hudState.update(state => ({ ...state, show_ip: newValue }));
    }
  }
</script>

<Section title="Display" bind:collapsed>
  <div class="display-controls">
    <div class="display-row">
      <Button active={$hudState.show_ip} onclick={handleToggleIp}>
        Show IP Address
      </Button>
      <span class="ip-display">{$hudState.ip_address || 'Not connected'}</span>
    </div>
  </div>
</Section>

<style>
  .display-controls {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-md);
  }

  .display-row {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
  }

  .ip-display {
    font-size: var(--font-size-base);
    opacity: 0.7;
    margin-left: var(--spacing-sm);
  }
</style>
