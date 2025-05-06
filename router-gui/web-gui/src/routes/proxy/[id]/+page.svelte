<script lang="ts">
    import { onMount } from "svelte";
    import { goto } from "$app/navigation";
    import { page } from "$app/stores";
    import { user } from "$lib/stores/userStore";
    import Button from "$lib/components/common/Button.svelte";
    import { proxyStore } from "$lib/stores/proxyStore";
    import { proxyActions } from "$lib/actions/proxyActions";
    import { gwnodeActions } from "$lib/actions/gwnodeActions";
    import type { Proxy, ProxyWithDomains, TlsDomain } from "$lib/types/proxy";
    import { apiToDomainConfigs } from "$lib/types/proxy";
    
    // Import our components
    import ProxyBasicInfo from "$lib/components/proxy/ProxyBasicInfo.svelte";
    import DomainConfiguration from "$lib/components/proxy/DomainConfiguration.svelte";
    import HighSpeedConfig from "$lib/components/proxy/HighSpeedConfig.svelte";
    
    // Authentication check
    let isLoggedIn = false;
    let isLoading = true;
    let isLoadingProxy = true;
    
    // Unsubscribe from user store
    const unsubAuthCheck = user.subscribe(value => {
        isLoggedIn = !!value;
        isLoading = false;
    });
    
    // UI Proxy interface
    interface UIProxy {
        id: string;
        title: string;
        listen: string;
        target: string;
        highSpeed: boolean;
        highSpeedAddr: string;
    }
    
    // Domain Config interface
    interface DomainConfig {
        id: string;
        domain: string;
        useTls: boolean;
        autoTls: boolean;
        certPem: string;
        certKey: string;
        proxy_id: string;
        gwnode_id?: string | null;
    }
    
    // TLS Domain interface for API
    interface TlsDomain {
        id?: string;
        sni: string;
        tls: boolean;
        tls_pem: string | null;
        tls_key: string | null;
        tls_autron: boolean;
        proxy_id: string; // Now required
        gwnode_id?: string | null;
    }
    
    // Gateway Node type
    interface GwNode {
        id: string;
        proxy_id: string;
        title: string;
        alt_target: string;
    }
    
    // Extended Proxy type to include tls_domains
    interface ExtendedProxy extends Proxy {
        tls_domains: TlsDomain[];
    }
    
    // Get the proxy ID from the URL
    const proxyId = $page.params.id;
    
    // Form state
    let proxy: UIProxy = {
        id: '',
        title: "",
        listen: "",
        target: "",
        highSpeed: false,
        highSpeedAddr: ""
    };
    
    // Domain configurations
    let domainConfigs: DomainConfig[] = [];
    
    // Helper state
    let isSaving = false;
    let errorMessage = "";
    let loadError = "";
    let gwNodes: GwNode[] = [];
    let loadingGwNodes = false;
    let errorLoadingGwNodes: string | null = null;
    
    // Convert API Proxy to UI format
    function apiToUiProxy(apiProxy: Proxy): { proxy: UIProxy, domains: DomainConfig[] } {
        // Create base proxy
        const uiProxy: UIProxy = {
            id: apiProxy.id || '',
            title: apiProxy.title,
            listen: apiProxy.addr_listen,
            target: apiProxy.addr_target || "",
            highSpeed: apiProxy.high_speed || false,
            highSpeedAddr: apiProxy.high_speed_addr || ""
        };
        
        // Handle domains
        const domains: DomainConfig[] = [];
        
        // Check if proxy has tls_domains property
        if ('tls_domains' in apiProxy && Array.isArray(apiProxy.tls_domains) && apiProxy.tls_domains.length > 0) {
            // Get domains from tls_domains property
            domains.push(...apiProxy.tls_domains.map((domain, index) => ({
                id: domain.id || `domain-${index}`,
                domain: domain.sni || "",
                useTls: !!domain.tls_pem || !!domain.tls_key || !!domain.tls_autron,
                autoTls: domain.tls_autron || false,
                certPem: domain.tls_pem || "",
                certKey: domain.tls_key || "",
                proxy_id: domain.proxy_id,
                gwnode_id: domain.gwnode_id || null
            })));
        }
        
        return { proxy: uiProxy, domains };
    }
    
    // Convert UI Proxy to API format
    function uiToApiProxy(uiProxy: UIProxy, domainConfigs: DomainConfig[]): ExtendedProxy {
        return {
            id: uiProxy.id,
            title: uiProxy.title,
            addr_listen: uiProxy.listen,
            addr_target: "", // Always send empty string as addr_target
            high_speed: uiProxy.highSpeed || false,
            high_speed_addr: uiProxy.highSpeedAddr || null,
            tls_domains: domainConfigs.length > 0 ? domainConfigs.map(config => {
                // For existing domains, keep proxy_id
                // For new domains (with generated IDs), don't set proxy_id
                const isNewDomain = config.id.startsWith('domain-');
                
                return {
                    sni: config.domain,
                    tls: config.useTls,
                    tls_pem: config.useTls ? config.certPem || null : null,
                    tls_key: config.useTls ? config.certKey || null : null,
                    tls_autron: config.useTls ? config.autoTls : false,
                    proxy_id: isNewDomain ? "" : (config.proxy_id || uiProxy.id),
                    gwnode_id: config.gwnode_id || null
                };
            }) : []
        };
    }
    
    // Load gateway nodes
    async function loadGwNodes() {
        loadingGwNodes = true;
        errorLoadingGwNodes = null;
        try {
            gwNodes = await gwnodeActions.getAvailableGwNodesForProxy(proxyId);
            console.log(`Loaded ${gwNodes.length} gateway nodes for proxy ${proxyId}`);
        } catch (error) {
            console.error("Failed to load gateway nodes:", error);
            errorLoadingGwNodes = "Failed to load gateway nodes";
            gwNodes = [];
        } finally {
            loadingGwNodes = false;
        }
    }
    
    // Load proxy data
    async function loadProxy() {
        try {
            isLoadingProxy = true;
            loadError = "";
            
            const apiProxyData = await proxyActions.getProxyById(proxyId);
            
            // Handle the new proxy with domains format
            if ('proxy' in apiProxyData && 'domains' in apiProxyData) {
                // New format with proxy and domains
                const proxyData = apiProxyData.proxy;
                
                // Convert to UI proxy format
                proxy = {
                    id: proxyData.id || '',
                    title: proxyData.title,
                    listen: proxyData.addr_listen,
                    target: proxyData.addr_target || "",
                    highSpeed: proxyData.high_speed || false,
                    highSpeedAddr: proxyData.high_speed_addr || ""
                };
                
                // Convert domains from API format to domain configs
                domainConfigs = apiToDomainConfigs(apiProxyData.domains);
                
                // Ensure all domains have proxy_id
                domainConfigs = domainConfigs.map(domain => ({
                    ...domain,
                    proxy_id: proxyData.id
                }));
            } else {
                // Legacy format, use old conversion
                const { proxy: loadedProxy, domains } = apiToUiProxy(apiProxyData as Proxy);
                proxy = loadedProxy;
                domainConfigs = domains;
            }
            
            // Load related data after proxy is loaded
            loadGwNodes();
        } catch (error) {
            console.error(`Failed to load proxy ${proxyId}:`, error);
            loadError = `Failed to load proxy: ${error instanceof Error ? error.message : String(error)}`;
        } finally {
            isLoadingProxy = false;
        }
    }
    
    // Save the proxy
    async function saveProxy() {
        // Validation - only check TLS domains if there are any
        if (domainConfigs.length > 0) {
            // Validate TLS domains if they have TLS enabled
            for (const config of domainConfigs) {
                if (config.useTls && !config.autoTls && (!config.certPem || !config.certKey)) {
                    errorMessage = `Domain ${config.domain} has TLS enabled but is missing certificate data`;
                    return;
                }
            }
        }
        
        if (proxy.highSpeed && !proxy.highSpeedAddr && gwNodes.length > 0) {
            errorMessage = "Please select a gateway node for high-speed mode";
            return;
        }
        
        try {
            isSaving = true;
            errorMessage = "";
            
            // Convert proxy object to API format
            const apiProxy = {
                id: proxy.id,
                title: proxy.title,
                addr_listen: proxy.listen,
                addr_target: "", // Empty string - server will handle this
                high_speed: proxy.highSpeed,
                high_speed_addr: proxy.highSpeedAddr || null
            };
            
            // Split domains into existing and new
            const domainsForApi = domainConfigs.map(config => {
                // For existing domains, keep proxy_id
                // For new domains (with generated IDs), remove proxy_id to let backend handle it
                const isNewDomain = config.id.startsWith('domain-');
                
                return {
                    ...config,
                    // Only keep proxy_id for existing domains
                    proxy_id: isNewDomain ? "" : config.proxy_id || proxy.id
                };
            });
            
            // Use proxyActions to save with the proxy and domains
            // proxyActions will handle the domain conversion
            await proxyActions.saveProxy(apiProxy, domainsForApi);
            
            // Go back to proxy list page
            goto('/proxy');
        } catch (error) {
            console.error('Error saving proxy:', error);
            errorMessage = `Failed to save proxy: ${error instanceof Error ? error.message : String(error)}`;
        } finally {
            isSaving = false;
        }
    }
    
    // Cancel and go back
    function cancel() {
        goto('/proxy');
    }
    
    // Cleanup and load data
    onMount(() => {
        // Redirect if not logged in
        if (!isLoading && !isLoggedIn) {
            goto('/');
            return;
        }
        
        // Load proxy data if we have an ID
        if (proxyId) {
            loadProxy();
        } else {
            // If no ID, redirect to proxy list
            goto('/proxy');
        }
        
        return () => {
            unsubAuthCheck();
        }
    });
</script>

{#if isLoading || isLoadingProxy}
    <div class="flex items-center justify-center h-screen">
        <div class="animate-spin rounded-full h-12 w-12 border-t-2 border-b-2 border-blue-600"></div>
    </div>
{:else if loadError}
    <div class="container mx-auto max-w-3xl px-4 py-8">
        <div class="bg-white dark:bg-gray-800 rounded-lg shadow-md p-6">
            <div class="flex flex-col items-center justify-center">
                <div class="text-red-600 dark:text-red-400 mb-4 text-lg font-medium">
                    {loadError}
                </div>
                <Button variant="primary" onClick={() => goto('/proxy')}>
                    Return to Proxy List
                </Button>
            </div>
        </div>
    </div>
{:else if isLoggedIn}
    <div class="container mx-auto max-w-3xl px-4 py-8">
        <div class="bg-white dark:bg-gray-800 rounded-lg shadow-md p-6">
            <div class="flex justify-between items-center mb-6">
                <h1 class="text-2xl font-bold text-gray-900 dark:text-white">Edit Proxy</h1>
                <Button variant="secondary" onClick={cancel}>
                    Cancel
                </Button>
            </div>
            
            {#if errorMessage}
                <div class="mb-4 p-3 bg-red-100 dark:bg-red-900/20 border border-red-200 dark:border-red-800 text-red-700 dark:text-red-400 rounded-md text-sm">
                    {errorMessage}
                </div>
            {/if}
            
            <form on:submit|preventDefault={saveProxy} class="space-y-6">
                <!-- Basic Proxy Info Component -->
                <ProxyBasicInfo 
                    bind:title={proxy.title}
                    bind:listen={proxy.listen}
                />
                
                <!-- Domain Configuration Component -->
                <DomainConfiguration bind:domainConfigs={domainConfigs} />
                
                <!-- High-Speed Mode Component -->
                <HighSpeedConfig 
                    bind:highSpeed={proxy.highSpeed}
                    bind:highSpeedAddr={proxy.highSpeedAddr}
                    bind:gwNodes
                    bind:loadingGwNodes
                    bind:errorLoadingGwNodes
                />
                
                <div class="flex justify-end pt-4">
                    <Button 
                        type="submit" 
                        variant="primary"
                        disabled={
                            isSaving || 
                            (proxy.highSpeed && !proxy.highSpeedAddr && gwNodes.length > 0)
                        }
                    >
                        {#if isSaving}
                            <span class="flex items-center">
                                <svg class="animate-spin -ml-1 mr-2 h-4 w-4 text-white" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                                    <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                                    <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                                </svg>
                                Updating...
                            </span>
                        {:else}
                            Update Proxy
                        {/if}
                    </Button>
                </div>
            </form>
        </div>
    </div>
{/if}