<script lang="ts">
    import { fade } from "svelte/transition";
    import type { GwNode } from "$lib/types/gwnode";
    import type { Proxy } from "$lib/types/proxy";
    import Button from "$lib/components/common/Button.svelte";
    import InputField from "$lib/components/common/InputField.svelte";
    import { proxyStore } from "$lib/stores/proxyStore";

    export let showModal: boolean = false;
    export let isEditMode: boolean = false;
    export let gwnode: GwNode = {
        id: "",
        title: "",
        proxy_id: "",
        proxyTitle: "",
        alt_target: "",
        source: "",
    };
    export let proxies: Proxy[] = [];
    export let onSave: () => void;
    export let onClose: () => void;

    // Additional proxy details
    let proxyListen: string = "";
    let proxyTls: boolean = false;
    let proxyDomain: string = "";

    // Convert proxies to options for select field
    $: proxyOptions = proxies.map((proxy) => ({
        value: proxy.id,
        label: proxy.title,
    }));

    // Fetch latest proxies when modal appears
    $: if (showModal) {
        fetchLatestProxies();
    }

    // Function to fetch the latest proxies from the API
    async function fetchLatestProxies() {
        try {
            await proxyStore.fetchProxies();
        } catch (error) {
            console.error("Error fetching proxies:", error);
        }
    }

    // Selected proxy information - track both proxy_id and proxies to ensure reactivity
    $: {
        if (gwnode.proxy_id && proxies.length > 0) {
            const selectedProxy = proxies.find((p) => p.id === gwnode.proxy_id);
            if (selectedProxy) {
                // Create a new object to ensure reactivity
                gwnode = {
                    ...gwnode,
                    proxyTitle: selectedProxy.title || "",
                };
                // Update additional proxy details
                proxyListen = selectedProxy.addr_listen || "";
                proxyTls = selectedProxy.tls || false;
                proxyDomain = selectedProxy.sni || "";
            }
        }
    }

    // Update gwnode when proxy_id changes
    $: if (gwnode.proxy_id) {
        const selectedProxy = proxies.find((p) => p.id === gwnode.proxy_id);
        if (selectedProxy) {
            gwnode = {
                ...gwnode,
                proxyTitle: selectedProxy.title || "",
            };
            // Update additional proxy details
            proxyListen = selectedProxy.addr_listen || "";
            proxyTls = selectedProxy.tls || false;
            proxyDomain = selectedProxy.sni || "";
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
        const selectedId = target.value;
        
        if (selectedId) {
            const selectedProxy = proxies.find((p) => p.id === selectedId);
            
            if (selectedProxy) {
                // Create a new object to ensure reactivity
                gwnode = {
                    ...gwnode,
                    proxy_id: selectedProxy.id,
                    proxyTitle: selectedProxy.title || "",
                };
                
                // Update additional proxy details
                proxyListen = selectedProxy.addr_listen || "";
                proxyTls = selectedProxy.tls || false;
                proxyDomain = selectedProxy.sni || "";
            }
        } else {
            // If no proxy is selected, clear the proxy-related fields
            gwnode = {
                ...gwnode,
                proxy_id: "",
                proxyTitle: "",
            };
            proxyListen = "";
            proxyTls = false;
            proxyDomain = "";
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
                        <label for="proxy_id" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                            Proxy<span class="text-red-500">*</span>
                        </label>
                        <select
                            id="proxy_id"
                            value={gwnode.proxy_id}
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

                    {#if gwnode.proxy_id}
                        <div class="rounded-md bg-gray-50 dark:bg-gray-800 p-3">
                            <h3
                                class="text-sm font-medium text-gray-700 dark:text-gray-300"
                            >
                                Selected Proxy
                            </h3>
                            <div
                                class="mt-1 text-sm text-gray-500 dark:text-gray-400"
                            >
                                <div class="mb-1">
                                    <span class="font-medium">Name:</span> {gwnode.proxyTitle || "Not specified"}
                                </div>
                                <div class="mb-1 font-mono text-xs">
                                    <span class="font-medium">Listen:</span> {proxyListen || "Not specified"}
                                </div>
                                <div class="mb-1 text-xs">
                                    <span class="font-medium">TLS:</span> {proxyTls ? "Enabled" : "Disabled"}
                                </div>
                                {#if proxyDomain}
                                <div class="text-xs">
                                    <span class="font-medium">Domain:</span> {proxyDomain}
                                </div>
                                {/if}
                            </div>
                        </div>
                    {/if}

                    <InputField
                        id="alt_target"
                        label="Target Address"
                        bind:value={gwnode.alt_target}
                        placeholder="IP:PORT"
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
