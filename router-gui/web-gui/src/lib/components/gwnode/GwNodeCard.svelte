<script lang="ts">
    import type { GwNode } from "$lib/types/gwnode";
    import { proxies } from "$lib/stores/proxyStore";

    // Props for the component
    export let gwnode: GwNode;
    export let onEdit: (gwnode: GwNode) => void;
    export let onDelete: (id: string) => void;

    // Proxy details
    let proxyListen = "";
    let proxyTls = false;
    let proxyDomain = "";
    let proxyTitle = "";

    // Get proxy details if proxy_id exists
    $: if (gwnode.proxy_id && $proxies) {
        const selectedProxy = $proxies.find((p) => p.id === gwnode.proxy_id);
        if (selectedProxy) {
            proxyListen = selectedProxy.addr_listen || "";
            proxyTls = selectedProxy.tls || false;
            proxyDomain = selectedProxy.sni || "";
            proxyTitle = selectedProxy.title || "";
        }
    }
</script>

<a href={`/gwnode/${gwnode.id}`} class="block relative">
    <div
        class="border border-gray-200 dark:border-gray-700 rounded-lg shadow-sm hover:shadow-md transition-shadow duration-300 p-4 relative"
    >
        <!-- Action buttons (positioned absolute top right) -->
        <div class="absolute top-2 right-2 flex space-x-2">
            <button
                on:click|stopPropagation|preventDefault={() => onEdit(gwnode)}
                class="p-1 text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200"
                aria-label="Edit"
            >
                <svg
                    xmlns="http://www.w3.org/2000/svg"
                    width="16"
                    height="16"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                >
                    <path
                        d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"
                    ></path>
                    <path
                        d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"
                    ></path>
                </svg>
            </button>
            <button
                on:click|stopPropagation|preventDefault={() =>
                    onDelete(gwnode.id)}
                class="p-1 text-red-500 hover:text-red-700 dark:text-red-400 dark:hover:text-red-300"
                aria-label="Delete"
            >
                <svg
                    xmlns="http://www.w3.org/2000/svg"
                    width="16"
                    height="16"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                >
                    <polyline points="3 6 5 6 21 6"></polyline>
                    <path
                        d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1-2 2v2"
                    ></path>
                    <line x1="10" y1="11" x2="10" y2="17"></line>
                    <line x1="14" y1="11" x2="14" y2="17"></line>
                </svg>
            </button>
        </div>

        <!-- Title -->
        <h3
            class="text-lg font-bold text-gray-900 dark:text-gray-100 mb-2 pr-16"
        >
            {gwnode.title}
        </h3>

        <!-- Proxy information -->
        <div class="mb-2">
            <h4 class="text-sm font-medium text-gray-700 dark:text-gray-300">
                Proxy Bind
            </h4>
            <table
                class="w-full text-sm text-left text-gray-700 dark:text-gray-300 mt-1"
            >
                <tbody>
                    <tr>
                        <td class="py-1 pr-2 italic w-20">Name:</td>
                        <td class="py-1">{proxyTitle || "Not specified"}</td>
                    </tr>
                    {#if proxyListen}
                        <tr>
                            <td class="py-1 pr-2 italic w-20">Listen:</td>
                            <td
                                class="py-1 font-mono text-xs text-gray-500 dark:text-gray-400"
                                >{proxyListen}</td
                            >
                        </tr>
                    {:else}
                        <tr>
                            <td class="py-1 pr-2 italic w-20">Listen:</td>
                            <td
                                class="py-1 font-mono text-xs text-gray-500 dark:text-gray-400"
                                >Not specified</td
                            >
                        </tr>
                    {/if}
                    {#if proxyDomain}
                        <tr>
                            <td class="py-1 pr-2 italic w-20">Domain:</td>
                            <td
                                class="py-1 font-mono text-xs text-gray-500 dark:text-gray-400"
                                >{proxyDomain}</td
                            >
                        </tr>
                    {:else}
                        <tr>
                            <td class="py-1 pr-2 italic w-20">Domain:</td>
                            <td
                                class="py-1 font-mono text-xs text-gray-500 dark:text-gray-400"
                                >-</td
                            >
                        </tr>
                    {/if}
                    {#if proxyTls}
                        <tr>
                            <td class="py-1 pr-2 italic w-20">Security:</td
                            >
                            <td
                                class="py-1 text-xs text-green-600 dark:text-green-400"
                                >TLS Enabled</td
                            >
                        </tr>
                    {:else}
                        <tr>
                            <td class="py-1 pr-2 italic w-20">Security:</td
                            >
                            <td
                                class="py-1 text-xs text-red-600 dark:text-red-400"
                                >Disabled</td
                            >
                        </tr>
                    {/if}
                </tbody>
            </table>
        </div>
        <hr class="my-2 border-gray-300 dark:border-gray-600" />
        <!-- Target -->
        <div class="mt-2">
            <h4 class="text-sm font-medium text-gray-700 dark:text-gray-300">
                Proxy Target
            </h4>
            <table
                class="w-full text-sm text-left text-gray-700 dark:text-gray-300 mt-1"
            >
                <tbody>
                    <tr>
                        <td class="py-1 pr-2 italic w-20">Address:</td>
                        <td class="py-1">
                            <span
                                class="font-mono bg-gray-100 dark:bg-gray-800 px-2 py-1 rounded"
                                >{gwnode.alt_target}</span
                            >
                        </td>
                    </tr>
                </tbody>
            </table>
        </div>
    </div>
</a>
