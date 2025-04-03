<script lang="ts">
  import { Moon, Sun } from 'lucide-svelte';
  import { isDarkMode } from '$lib/stores/theme';

  // Set the initial theme when the component is mounted
  import { onMount } from 'svelte';

  onMount(() => {
    isDarkMode.initialize();
  });
</script>

<button
  aria-label="Toggle theme"
  class="p-2 rounded-full hover:bg-github-light-border dark:hover:bg-github-dark-border transition-colors"
  on:click={() => isDarkMode.toggleTheme()}
>
  {#if $isDarkMode}
    <Sun class="text-github-dark-text" size={20} />
  {:else}
    <Moon class="text-github-light-text" size={20} />
  {/if}
</button>