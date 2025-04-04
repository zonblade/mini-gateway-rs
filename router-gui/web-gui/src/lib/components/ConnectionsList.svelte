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
    <h3 class="text-lg font-medium">Connections</h3>
    {#if !isAddingConnection}
      <button
        on:click={() => isAddingConnection = true}
        class="px-2 py-1 text-sm bg-[#238636] hover:bg-[#2ea043] text-white rounded"
      >
        Add New
      </button>
    {/if}
  </div>
  
  {#if isAddingConnection}
    <div class="mb-4 p-3 border border-gray-200 dark:border-gray-700 rounded-md">
      <h4 class="text-sm font-medium mb-2">New Connection</h4>
      <div class="space-y-2">
        <div>
          <label for="name" class="block text-xs font-medium mb-1">Name</label>
          <input
            type="text"
            id="name"
            bind:value={newConnection.name}
            class="w-full px-2 py-1 text-sm border border-gray-300 dark:border-gray-700 rounded bg-white dark:bg-[#0d1117] focus:outline-none focus:ring-1 focus:ring-[#1f6feb] focus:border-[#1f6feb]"
            placeholder="My Connection"
          />
        </div>
        <div>
          <label for="host" class="block text-xs font-medium mb-1">Host</label>
          <input
            type="text"
            id="host"
            bind:value={newConnection.host}
            class="w-full px-2 py-1 text-sm border border-gray-300 dark:border-gray-700 rounded bg-white dark:bg-[#0d1117] focus:outline-none focus:ring-1 focus:ring-[#1f6feb] focus:border-[#1f6feb]"
            placeholder="localhost"
          />
        </div>
        <div>
          <label for="port" class="block text-xs font-medium mb-1">Port</label>
          <input
            type="number"
            id="port"
            bind:value={newConnection.port}
            class="w-full px-2 py-1 text-sm border border-gray-300 dark:border-gray-700 rounded bg-white dark:bg-[#0d1117] focus:outline-none focus:ring-1 focus:ring-[#1f6feb] focus:border-[#1f6feb]"
            placeholder="8080"
          />
        </div>
        <div class="flex justify-end space-x-2 mt-3">
          <button
            on:click={() => isAddingConnection = false}
            class="px-2 py-1 text-xs border border-gray-300 dark:border-gray-700 rounded"
          >
            Cancel
          </button>
          <button
            on:click={handleAddConnection}
            class="px-2 py-1 text-xs bg-[#238636] hover:bg-[#2ea043] text-white rounded"
          >
            Save
          </button>
        </div>
      </div>
    </div>
  {/if}
  
  <div class="flex-1 overflow-y-auto">
    <div class="space-y-2">
      {#each $connections as connection}
        <div 
          class="p-3 rounded-md cursor-pointer transition-colors {selectedConnectionId === connection.id ? 'bg-[#1f6feb]/10 border-[#1f6feb] border' : 'hover:bg-gray-100 dark:hover:bg-gray-800 border border-gray-200 dark:border-gray-700'}"
          on:click={() => selectedConnectionId = connection.id}
        >
          <div class="flex justify-between items-start">
            <div>
              <div class="font-medium">{connection.name}</div>
              <div class="text-sm text-gray-500 dark:text-gray-400">{connection.host}:{connection.port}</div>
            </div>
            {#if $connections.length > 1}
              <button 
                on:click|stopPropagation={() => handleRemoveConnection(connection.id)}
                class="text-gray-500 hover:text-red-500 dark:text-gray-400 dark:hover:text-red-400"
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
</div>