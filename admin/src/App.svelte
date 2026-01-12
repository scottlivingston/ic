<script lang="ts">
  import { onMount } from 'svelte';
  import { hudState, phrasesEditMode } from './lib/stores';
  import { fetchHudStatus, toggleIpHud } from './lib/api';

  import EffectsPanel from './lib/components/EffectsPanel.svelte';
  import VolumeControl from './lib/components/VolumeControl.svelte';
  import WifiManager from './lib/components/WifiManager.svelte';
  import SpeechPanel from './lib/components/SpeechPanel.svelte';
  import Button from './lib/components/ui/Button.svelte';

  const STORAGE_KEY = 'ic-admin-panels';
  const saved = JSON.parse(localStorage.getItem(STORAGE_KEY) || '{}');

  let showCRT = $state(saved.crt ?? false);
  let showVol = $state(saved.vol ?? false);
  let showWifi = $state(saved.wifi ?? false);

  $effect(() => {
    localStorage.setItem(STORAGE_KEY, JSON.stringify({
      crt: showCRT,
      vol: showVol,
      wifi: showWifi
    }));
  });

  async function handleToggleIp() {
    const newValue = !$hudState.show_ip;
    const success = await toggleIpHud(newValue);
    if (success) {
      hudState.update(state => ({ ...state, show_ip: newValue }));
    }
  }

  onMount(async () => {
    const status = await fetchHudStatus();
    hudState.set(status);
  });
</script>

<header class="header">
  <h1>IC Admin</h1>
  <nav class="header-controls">
    <Button variant="icon" active={showCRT} onclick={() => showCRT = !showCRT}>CRT</Button>
    <Button variant="icon" active={showVol} onclick={() => showVol = !showVol}>VOL</Button>
    <Button variant="icon" active={showWifi} onclick={() => showWifi = !showWifi}>WIFI</Button>
    <Button variant="icon" active={$phrasesEditMode} onclick={() => phrasesEditMode.update(v => !v)}>EDIT</Button>
    <Button variant="icon" active={$hudState.show_ip} onclick={handleToggleIp}>IP</Button>
  </nav>
</header>

{#if showCRT}
  <EffectsPanel />
{/if}
{#if showVol}
  <VolumeControl />
{/if}
{#if showWifi}
  <WifiManager />
{/if}

<SpeechPanel />

<style>
  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--spacing-2xl);
  }

  .header h1 {
    margin-bottom: 0;
  }

  .header-controls {
    display: flex;
    gap: var(--spacing-sm);
  }
</style>
