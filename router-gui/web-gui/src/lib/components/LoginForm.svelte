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
    class="mb-4 p-3 bg-red-100 dark:bg-red-900/20 border border-red-200 dark:border-red-800 text-red-700 dark:text-red-400 rounded-md text-sm"
  >
    {errorMessage}
  </div>
{/if}

<form on:submit|preventDefault={onSubmit} class="space-y-6">
  <div>
    <label for="username" class="block text-sm font-medium mb-1"
      >Username</label
    >
    <input
      type="text"
      id="username"
      bind:value={username}
      class="w-full px-3 py-2 border border-gray-300 dark:border-gray-700 rounded-md shadow-sm bg-white dark:bg-[#0d1117] focus:outline-none focus:ring-2 focus:ring-[#1f6feb] focus:border-[#1f6feb] transition-colors"
      placeholder="Enter your username"
    />
  </div>

  <div>
    <label for="password" class="block text-sm font-medium mb-1"
      >Password</label
    >
    <input
      type="password"
      id="password"
      bind:value={password}
      class="w-full px-3 py-2 border border-gray-300 dark:border-gray-700 rounded-md shadow-sm bg-white dark:bg-[#0d1117] focus:outline-none focus:ring-2 focus:ring-[#1f6feb] focus:border-[#1f6feb] transition-colors"
      placeholder="Enter your password"
    />
  </div>

  <div class="mt-12">
    <button
      type="submit"
      disabled={isSubmitting}
      class="w-full py-2 px-4 bg-[#238636] hover:bg-[#2ea043] disabled:bg-[#238636]/70 disabled:cursor-not-allowed text-white rounded-md transition-colors focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-[#238636]"
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