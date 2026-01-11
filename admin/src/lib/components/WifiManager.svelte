<script lang="ts">
  import { wifiNetworks, wifiScanning, wifiConnecting, selectedNetwork, hudState } from '../stores';
  import { scanWifi, connectWifi, forgetWifi, fetchHudStatus } from '../api';
  import Section from './ui/Section.svelte';
  import Button from './ui/Button.svelte';
  import TextInput from './ui/TextInput.svelte';

  let collapsed = $state(false);
  let password = $state('');

  async function handleScan() {
    wifiScanning.set(true);
    const networks = await scanWifi();
    wifiNetworks.set(networks);
    wifiScanning.set(false);
  }

  async function handleConnect() {
    if (!$selectedNetwork) return;

    wifiConnecting.set(true);
    const result = await connectWifi($selectedNetwork, password);

    if (result.success) {
      alert(`Connected to ${$selectedNetwork}!\nIP: ${result.ip_address || 'unknown'}`);
      selectedNetwork.set(null);
      password = '';
      const status = await fetchHudStatus();
      hudState.set(status);
    } else {
      alert(`Failed to connect: ${result.error || 'Unknown error'}`);
    }

    wifiConnecting.set(false);
  }

  async function handleForget(ssid: string) {
    if (confirm(`Forget network "${ssid}"?`)) {
      const result = await forgetWifi(ssid);
      if (result.success) {
        await handleScan();
      } else {
        alert(`Failed to forget network: ${result.error || 'Unknown error'}`);
      }
    }
  }

  function selectNetwork(ssid: string, inUse: boolean) {
    if (!inUse) {
      selectedNetwork.set(ssid);
    }
  }

  function cancelConnect() {
    selectedNetwork.set(null);
    password = '';
  }
</script>

<Section title="WiFi" bind:collapsed>
  <div class="wifi-controls">
    <Button onclick={handleScan} disabled={$wifiScanning}>
      {$wifiScanning ? 'Scanning...' : 'Scan Networks'}
    </Button>

    <div class="wifi-networks">
      {#if $wifiNetworks.length === 0}
        <div class="wifi-empty">No networks found. Click Scan to search.</div>
      {:else}
        {#each $wifiNetworks as network}
          <button
            type="button"
            class="wifi-network"
            class:wifi-connected={network.in_use}
            onclick={() => selectNetwork(network.ssid, network.in_use)}
          >
            <div class="wifi-info">
              <span class="wifi-ssid">
                {network.ssid}{network.in_use ? ' (connected)' : ''}
              </span>
              <span class="wifi-signal">{network.signal}% {network.security}</span>
            </div>
            {#if network.in_use}
              <Button variant="small" onclick={(e) => { e.stopPropagation(); handleForget(network.ssid); }}>
                Forget
              </Button>
            {/if}
          </button>
        {/each}
      {/if}
    </div>

    {#if $selectedNetwork}
      <div class="wifi-connect-form">
        <div class="wifi-selected">Connecting to: {$selectedNetwork}</div>
        <TextInput
          type="password"
          placeholder="Password"
          bind:value={password}
        />
        <div class="wifi-form-buttons">
          <Button onclick={handleConnect} disabled={$wifiConnecting}>
            {$wifiConnecting ? 'Connecting...' : 'Connect'}
          </Button>
          <Button onclick={cancelConnect}>
            Cancel
          </Button>
        </div>
      </div>
    {/if}
  </div>
</Section>

<style>
  .wifi-controls {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-md);
  }

  .wifi-networks {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs);
    max-height: 200px;
    overflow-y: auto;
  }

  .wifi-network {
    display: flex;
    justify-content: space-between;
    align-items: center;
    width: 100%;
    padding: var(--spacing-sm) var(--spacing-md);
    border: var(--border);
    background: var(--color-bg);
    color: var(--color-primary);
    font-family: var(--font-family);
    font-size: var(--font-size-base);
    text-align: left;
    cursor: pointer;
  }

  .wifi-network:hover {
    background: var(--color-bg-hover);
  }

  .wifi-connected {
    background: var(--color-bg-active);
    cursor: default;
  }

  .wifi-info {
    display: flex;
    flex-direction: column;
    flex: 1;
  }

  .wifi-ssid {
    font-size: var(--font-size-base);
  }

  .wifi-signal {
    font-size: var(--font-size-small);
    opacity: 0.7;
  }

  .wifi-empty {
    font-size: var(--font-size-base);
    opacity: 0.5;
    padding: var(--spacing-sm);
    text-align: center;
  }

  .wifi-connect-form {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
    padding: var(--spacing-md);
    border: var(--border);
    background: var(--color-bg-subtle);
  }

  .wifi-selected {
    font-size: var(--font-size-base);
    opacity: 0.8;
  }

  .wifi-form-buttons {
    display: flex;
    gap: var(--spacing-sm);
  }

  .wifi-form-buttons :global(.btn) {
    flex: 1;
  }
</style>
