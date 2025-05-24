<script lang="ts">
  // Form state management
  export let username: string = "";
  export let password: string = "";
  export let isSubmitting: boolean = false;
  export let errorMessage: string = "";
  export let onSubmit: () => void;
</script>

{#if errorMessage}
  <div
    class="mb-4 p-3 bg-red-50 dark:bg-red-900/10 border-l-2 border-red-500 dark:border-red-700 text-red-700 dark:text-red-400 text-sm"
  >
    {errorMessage}
  </div>
{/if}

<form on:submit|preventDefault={onSubmit} class="space-y-6">
  <div>
    <label for="username" class="block text-sm mb-1">Username</label>
    <input
      type="text"
      id="username"
      bind:value={username}
      class="w-full px-3 py-2 border-b border-gray-300 dark:border-gray-700 bg-transparent focus:outline-none focus:border-gray-500 dark:focus:border-gray-400 transition-colors"
      placeholder="Enter your username"
    />
  </div>

  <div>
    <label for="password" class="block text-sm mb-1">Password</label>
    <input
      type="password"
      id="password"
      bind:value={password}
      class="w-full px-3 py-2 border-b border-gray-300 dark:border-gray-700 bg-transparent focus:outline-none focus:border-gray-500 dark:focus:border-gray-400 transition-colors"
      placeholder="Enter your password"
    />
  </div>

  <div class="mt-12">
    <button
      type="submit"
      disabled={isSubmitting}
      class="w-full py-2 px-4 bg-gray-800 hover:bg-gray-700 dark:bg-gray-700 dark:hover:bg-gray-600 disabled:opacity-50 disabled:cursor-not-allowed text-white transition-colors focus:outline-none"
    >
      {#if isSubmitting}
        <span class="flex justify-center items-center">
          <svg
            class="animate-spin -ml-1 mr-2 h-4 w-4 text-white"
            xmlns="http://www.w3.org/2000/svg"
            fill="none"
            viewBox="0 0 24 24"
          >
            <circle
              class="opacity-25"
              cx="12"
              cy="12"
              r="10"
              stroke="currentColor"
              stroke-width="4"
            ></circle>
            <path
              class="opacity-75"
              fill="currentColor"
              d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
            ></path>
          </svg>
          Logging in...
        </span>
      {:else}
        Sign in
      {/if}
    </button>
  </div>
</form>