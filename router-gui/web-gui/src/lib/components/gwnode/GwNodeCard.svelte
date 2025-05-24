<script lang="ts">
    import type { GwNode } from "$lib/types/gwnode";
    import { proxies } from "$lib/stores/proxyStore";
    import { Edit, Trash2 } from 'lucide-svelte';

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
        const selectedProxy = $proxies.find((p) => p.proxy.id === gwnode.proxy_id);
        if (selectedProxy) {
            proxyListen = selectedProxy.proxy.addr_listen || "";
            
            // If gwnode has a domain_id, find the matching domain
            if (gwnode.domain_id && selectedProxy.domains) {
                const domain = selectedProxy.domains.find(d => d.id === gwnode.domain_id);
                if (domain) {
                    proxyTls = domain.tls || false;
                    proxyDomain = domain.sni || "";
                } else {
                    proxyTls = false;
                    proxyDomain = gwnode.domain_name || ""; // Fallback to domain_name if domain not found
                }
            } else {
                proxyTls = false;
                proxyDomain = "";
            }
            
            proxyTitle = selectedProxy.proxy.title || "";
        }
    }
</script>

<a href={`/gwnode/${gwnode.id}`} class="block relative">
    <div
        class="border border-gray-200 dark:border-gray-700 hover:bg-white/40 hover:dark:bg-gray-800/40 hover:border-gray-300 dark:hover:border-gray-600 duration-200 p-4 relative"
    >
        <!-- Action buttons (positioned absolute top right) -->
        <div class="absolute top-5 right-2 flex space-x-2">
            <button
                on:click|stopPropagation|preventDefault={() => onEdit(gwnode)}
                class="p-1 text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200"
                aria-label="Edit"
            >
                <Edit size={16} />
            </button>
            <button
                on:click|stopPropagation|preventDefault={() =>
                    onDelete(gwnode.id)}
                class="p-1 text-red-500 hover:text-red-700 dark:text-red-400 dark:hover:text-red-300"
                aria-label="Delete"
            >
                <Trash2 size={16} />
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
