<script lang="ts">
    import { fade } from "svelte/transition";
    import type { GwNode } from "$lib/stores/gwnodeStore";
    import type { Proxy } from "$lib/stores/proxyStore";
    import Button from "$lib/components/common/Button.svelte";
    import InputField from "$lib/components/common/InputField.svelte";

    export let showModal: boolean = false;
    export let isEditMode: boolean = false;
    export let gwnode: GwNode = {
        id: 0,
        title: "",
        proxyId: 0,
        proxyTitle: "",
        proxyListen: "",
        target: "",
    };
    export let proxies: Proxy[] = [];
    export let onSave: () => void;
    export let onClose: () => void;

    // Convert proxies to options for select field
    $: proxyOptions = proxies.map((proxy) => ({
        value: proxy.id,
        label: proxy.title,
    }));

    // Selected proxy information - track both proxyId and proxies to ensure reactivity
    $: {
        if (gwnode.proxyId && proxies.length > 0) {
            const selectedProxy = proxies.find((p) => p.id === gwnode.proxyId);
            if (selectedProxy) {
                // Create a new object to ensure reactivity
                gwnode = {
                    ...gwnode,
                    proxyTitle: selectedProxy.title || "",
                    proxyListen: selectedProxy.listen || "",
                };
                console.log("Updated proxy details:", gwnode); // Add logging for debugging
            }
        }
    }

    // Update gwnode when proxyId changes
    $: if (gwnode.proxyId) {
        const selectedProxy = proxies.find((p) => p.id === gwnode.proxyId);
        if (selectedProxy) {
            gwnode = {
                ...gwnode,
                proxyTitle: selectedProxy.title || "",
                proxyListen: selectedProxy.listen || "",
            };
        }
    }

    // Handle ESC key to close modal
    function handleKeydown(event: KeyboardEvent) {
        if (event.key === "Escape") {
            onClose();
        }
    }

    // Keep events from propagating outside the modal
    function handleModalKeyDown(event: KeyboardEvent) {
        event.stopPropagation();
    }

    // Handle proxy selection
    function handleProxyChange(event: Event) {
        const target = event.target as HTMLSelectElement;
        const selectedId = parseInt(target.value);
        const selectedProxy = proxies.find(p => p.id === selectedId);
        
        if (selectedProxy) {
            gwnode = {
                ...gwnode,
                proxyId: selectedProxy.id,
                proxyTitle: selectedProxy.title,
                proxyListen: selectedProxy.listen
            };
        }
    }
</script>

{#if showModal}
    <div
        class="fixed inset-0 bg-black/30 backdrop-blur-md bg-opacity-50 flex items-center justify-center z-50"
        transition:fade={{ duration: 200 }}
        on:keydown={handleKeydown}
        role="presentation"
    >
        <div
            class="bg-white dark:bg-[#161b22] backdrop-blur-md rounded-lg shadow-xl max-w-md w-full mx-4"
            on:click|stopPropagation
            on:keydown={handleModalKeyDown}
            role="dialog"
            aria-labelledby="modal-title"
            aria-modal="true"
            tabindex="-1"
        >
            <div class="p-6">
                <div class="flex justify-between items-center mb-4">
                    <h2 id="modal-title" class="text-xl font-bold">
                        {isEditMode ? "Edit Gateway Node" : "Add Gateway Node"}
                    </h2>
                    <button
                        on:click={onClose}
                        aria-label="Close"
                        class="text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200"
                    >
                        <svg
                            xmlns="http://www.w3.org/2000/svg"
                            width="20"
                            height="20"
                            viewBox="0 0 24 24"
                            fill="none"
                            stroke="currentColor"
                            stroke-width="2"
                            stroke-linecap="round"
                            stroke-linejoin="round"
                        >
                            <line x1="18" y1="6" x2="6" y2="18"></line>
                            <line x1="6" y1="6" x2="18" y2="18"></line>
                        </svg>
                    </button>
                </div>

                <form on:submit|preventDefault={onSave} class="space-y-4">
                    <InputField
                        id="title"
                        label="Title"
                        bind:value={gwnode.title}
                        placeholder="My Gateway Node"
                        required={true}
                    />

                    <div>
                        <label for="proxyId" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                            Proxy<span class="text-red-500">*</span>
                        </label>
                        <select
                            id="proxyId"
                            value={gwnode.proxyId}
                            on:change={handleProxyChange}
                            class="w-full p-2 rounded-md border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                            required
                        >
                            <option value="">Select a proxy</option>
                            {#each proxies as proxy}
                                <option value={proxy.id}>{proxy.title}</option>
                            {/each}
                        </select>
                    </div>

                    {#if gwnode.proxyId}
                        <div class="rounded-md bg-gray-50 dark:bg-gray-800 p-3">
                            <h3
                                class="text-sm font-medium text-gray-700 dark:text-gray-300"
                            >
                                Selected Proxy
                            </h3>
                            <div
                                class="mt-1 text-sm text-gray-500 dark:text-gray-400"
                            >
                                <div>
                                    {gwnode.proxyTitle || "Not specified"}
                                </div>
                                <div class="font-mono text-xs">
                                    {gwnode.proxyListen || "Not specified"}
                                </div>
                            </div>
                        </div>
                    {/if}

                    <InputField
                        id="target"
                        label="Target (IP:Port)"
                        bind:value={gwnode.target}
                        placeholder="Example: 192.168.1.10:8080"
                        required={true}
                    />

                    <div class="flex justify-end space-x-2 pt-4">
                        <Button variant="secondary" onClick={onClose}>
                            Cancel
                        </Button>
                        <Button type="submit" variant="primary">
                            {isEditMode ? "Update" : "Create"}
                        </Button>
                    </div>
                </form>
            </div>
        </div>
    </div>
{/if}
