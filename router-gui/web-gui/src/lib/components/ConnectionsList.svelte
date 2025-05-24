<script lang="ts">
  import { connections } from "$lib/stores/connectionsStore";
  import type { Connection } from "$lib/stores/connectionsStore";
  
  export let selectedConnectionId: string = "";
  
  // Use auto-subscription with the $ prefix
  $: connectionsList = $connections;
  $: if (!selectedConnectionId && connectionsList.length > 0) {
    selectedConnectionId = connectionsList[0].id;
  }
  
  // Form for adding a new connection
  let isAddingConnection = false;
  let newConnection = {
    name: "",
    host: "",
    port: 8080
  };
  
  function handleAddConnection() {
    if (!newConnection.name || !newConnection.host) {
      return;
    }
    
    connections.addConnection(newConnection);
    newConnection = { name: "", host: "", port: 8080 };
    isAddingConnection = false;
  }
  
  function handleRemoveConnection(id: string) {
    connections.removeConnection(id);
    if (selectedConnectionId === id && $connections.length > 0) {
      selectedConnectionId = $connections[0].id;
    }
  }
</script>

<div class="h-full flex flex-col">
  <div class="flex justify-between items-center mb-4">
    <h3 class="text-lg font-normal">Connections</h3>
    {#if !isAddingConnection}
      <button
        on:click={() => isAddingConnection = true}
        class="px-2 py-1 text-sm bg-gray-100 hover:bg-gray-200 dark:bg-gray-800 dark:hover:bg-gray-700 text-gray-800 dark:text-gray-200"
      >
        Add New
      </button>
    {/if}
  </div>
  
  <div class="flex-1 overflow-y-auto">
    <div class="space-y-2">
      {#each $connections as connection}
        <div 
          class="p-3 cursor-pointer transition-colors {selectedConnectionId === connection.id ? 'bg-gray-100 dark:bg-gray-800 border-l-2 border-gray-500 dark:border-gray-400' : 'hover:bg-gray-50 dark:hover:bg-gray-900 border-l-2 border-transparent'}"
          on:click={() => selectedConnectionId = connection.id}
          on:keydown={(e) => e.key === 'Enter' && (selectedConnectionId = connection.id)}
          tabindex="0"
          role="button"
          aria-pressed={selectedConnectionId === connection.id}
        >
          <div class="flex justify-between items-start">
            <div>
              <div class="font-normal">{connection.name}</div>
              <div class="text-sm text-gray-500 dark:text-gray-400">{connection.host}:{connection.port}</div>
            </div>
            {#if $connections.length > 1}
              <button 
                on:click|stopPropagation={() => handleRemoveConnection(connection.id)}
                class="text-gray-400 hover:text-gray-600 dark:text-gray-500 dark:hover:text-gray-300"
                aria-label="Remove connection {connection.name}"
              >
                <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
                  <path fill-rule="evenodd" d="M9 2a1 1 0 00-.894.553L7.382 4H4a1 1 0 000 2v10a2 2 0 002 2h8a2 2 0 002-2V6a1 1 0 100-2h-3.382l-.724-1.447A1 1 0 0011 2H9zM7 8a1 1 0 012 0v6a1 1 0 11-2 0V8zm5-1a1 1 0 00-1 1v6a1 1 0 102 0V8a1 1 0 00-1-1z" clip-rule="evenodd" />
                </svg>
              </button>
            {/if}
          </div>
        </div>
      {/each}
    </div>
  </div>
  
  <!-- Popup Modal for Adding Connection -->
  {#if isAddingConnection}
    <div class="fixed inset-0 z-50 flex items-center justify-center">
      <!-- Backdrop with blur effect -->
      <div 
        class="absolute inset-0 bg-gray-500/30 dark:bg-gray-900/50 backdrop-blur-sm transition-opacity"
        on:click={() => isAddingConnection = false}
        on:keydown={(e) => e.key === 'Escape' && (isAddingConnection = false)}
        tabindex="-1"
      ></div>
      
      <!-- Modal Content -->
      <div 
        class="relative bg-white dark:bg-[#1a1a1a] border border-gray-200 dark:border-gray-800 w-full max-w-md p-5 transition-all"
        role="dialog"
        aria-modal="true"
      >
        <div class="flex justify-between items-center mb-4">
          <h4 class="text-base font-normal">New Connection</h4>
          <button 
            on:click={() => isAddingConnection = false}
            class="text-gray-400 hover:text-gray-600 dark:text-gray-500 dark:hover:text-gray-300"
            aria-label="Close modal"
          >
            <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
              <path fill-rule="evenodd" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" clip-rule="evenodd" />
            </svg>
          </button>
        </div>
        
        <div class="space-y-4">
          <div>
            <label for="name" class="block text-sm mb-1">Name</label>
            <input
              type="text"
              id="name"
              bind:value={newConnection.name}
              class="w-full px-3 py-2 border-b border-gray-300 dark:border-gray-700 bg-transparent focus:outline-none focus:border-gray-500 dark:focus:border-gray-400"
              placeholder="My Connection"
            />
          </div>
          <div>
            <label for="host" class="block text-sm mb-1">Host</label>
            <input
              type="text"
              id="host"
              bind:value={newConnection.host}
              class="w-full px-3 py-2 border-b border-gray-300 dark:border-gray-700 bg-transparent focus:outline-none focus:border-gray-500 dark:focus:border-gray-400"
              placeholder="localhost"
            />
          </div>
          <div>
            <label for="port" class="block text-sm mb-1">Port</label>
            <input
              type="number"
              id="port"
              bind:value={newConnection.port}
              class="w-full px-3 py-2 border-b border-gray-300 dark:border-gray-700 bg-transparent focus:outline-none focus:border-gray-500 dark:focus:border-gray-400"
              placeholder="8080"
            />
          </div>
          <div class="flex justify-end space-x-3 mt-6">
            <button
              on:click={() => isAddingConnection = false}
              class="px-3 py-2 text-sm text-gray-600 dark:text-gray-400"
            >
              Cancel
            </button>
            <button
              on:click={handleAddConnection}
              class="px-3 py-2 text-sm bg-gray-800 hover:bg-gray-700 dark:bg-gray-700 dark:hover:bg-gray-600 text-white"
            >
              Save
            </button>
          </div>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  /* Animation for modal fade-in */
  .fixed {
    animation: fadeIn 0.2s ease-out;
  }
  
  @keyframes fadeIn {
    from { opacity: 0; }
    to { opacity: 1; }
  }
</style>