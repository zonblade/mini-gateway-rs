<script>
  import "../app.css";
  import ThemeToggle from '$lib/components/ThemeToggle.svelte';
</script>

<div class="min-h-screen bg-github-light-bg dark:bg-github-dark-bg text-github-light-text dark:text-github-dark-text">
  <header class="container mx-auto py-4 px-4 flex justify-between items-center">
    <div class="text-2xl font-bold">Router GUI</div>
    <ThemeToggle />
  </header>
  
  <main class="container mx-auto px-4 py-8">
    <slot />
  </main>
</div>