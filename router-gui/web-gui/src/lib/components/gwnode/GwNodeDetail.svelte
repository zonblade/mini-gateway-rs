<script lang="ts">
    import type { GwNode } from "$lib/types/gwnode";
    import GatewayTableManager from "$lib/components/gateway/GatewayTableManager.svelte";
    import { proxies } from "$lib/stores/proxyStore";
    
    export let gwnode: GwNode;
    
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

<div class="bg-white dark:bg-gray-800 border-1 border-gray-200 dark:border-gray-700 overflow-hidden">
    <div class="p-6">
        <h1 class="text-2xl font-bold text-gray-900 dark:text-white mb-4">
            {gwnode.title}
        </h1>

        <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
            <!-- Proxy Information -->
            <div class="space-y-3">
                <h3 class="text-lg font-medium text-gray-700 dark:text-gray-300">
                    Proxy Information
                </h3>
                <div class="bg-gray-50 dark:bg-gray-700 p-4 ">
                    <div class="mb-1">
                        <span class="font-medium">Name:</span>
                        {proxyTitle || 'Not specified'}
                    </div>
                    <div class="mb-1">
                        <span class="font-medium">Listen:</span>
                        <code class="text-sm bg-gray-200 dark:bg-gray-600 px-1 py-0.5 "
                            >{proxyListen}</code>
                    </div>
                    {#if proxyTls}
                        <div class="mb-1">
                            <span class="font-medium">TLS:</span>
                            <span class="text-green-600 dark:text-green-400">Enabled</span>
                        </div>
                    {/if}
                    {#if proxyDomain}
                        <div>
                            <span class="font-medium">Domain:</span>
                            <code class="text-sm bg-gray-200 dark:bg-gray-600 px-1 py-0.5 "
                                >{proxyDomain}</code>
                        </div>
                    {/if}
                </div>
            </div>

            <!-- Target Information -->
            <div class="space-y-3">
                <h3 class="text-lg font-medium text-gray-700 dark:text-gray-300">
                    Target Information
                </h3>
                <div class="bg-gray-50 dark:bg-gray-700 p-4 ">
                    <div>
                        <span class="font-medium">Address:</span>
                        <code class="text-sm bg-gray-200 dark:bg-gray-600 px-1 py-0.5 "
                            >{gwnode.alt_target}</code>
                    </div>
                </div>
            </div>
        </div>
    </div>
    
    <!-- Gateway Routing Rules Table -->
    <GatewayTableManager 
        gwnodeId={gwnode.id}
        gwnodeTitle={gwnode.title}
    />
</div>