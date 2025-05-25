<script lang="ts">
    import { onMount } from "svelte";
    import { goto } from "$app/navigation";
    import { user } from "$lib/stores/userStore";
    import Button from "$lib/components/common/Button.svelte";
    import { proxyStore } from "$lib/stores/proxyStore";
    import { gwnodeActions } from "$lib/actions/gwnodeActions";
    import { proxyActions } from "$lib/actions/proxyActions";
    import type { Proxy } from "$lib/types/proxy";
    
    // Import our new components
    import ProxyBasicInfo from "$lib/components/proxy/ProxyBasicInfo.svelte";
    import DomainConfiguration from "$lib/components/proxy/DomainConfiguration.svelte";
    import HighSpeedConfig from "$lib/components/proxy/HighSpeedConfig.svelte";
    import LoadingSpinner from "$lib/components/common/LoadingSpinner.svelte";
    
    // Authentication check
    let isLoggedIn = false;
    let isLoading = true;
    
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
    }
    
    // TLS Domain interface for API
    interface TlsDomain {
        id?: string;
        sni: string;
        tls: boolean;
        tls_pem: string | null;
        tls_key: string | null;
        tls_autron: boolean;
        proxy_id: string; // Required
        gwnode_id?: string | null;
    }
    
    // Extended Proxy type to include tls_domains
    interface ExtendedProxy extends Proxy {
        tls_domains: TlsDomain[];
    }
    
    // Gateway Node type
    interface GwNode {
        id: string;
        proxy_id: string;
        title: string;
        alt_target: string;
    }
    
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
    
    // Selected gateway node for high-speed mode
    let selectedGwNodeId: string = "";
    
    // Helper state
    let isSaving = false;
    let errorMessage = "";
    let gwNodes: GwNode[] = [];
    let loadingGwNodes = false;
    let errorLoadingGwNodes: string | null = null;
    
    // Load gateway nodes
    async function loadGwNodes() {
        loadingGwNodes = true;
        errorLoadingGwNodes = null;
        try {
            gwNodes = await gwnodeActions.getAvailableGwNodesForProxy();
            console.log(`Loaded ${gwNodes.length} unbound gateway nodes for new proxy`);
        } catch (error) {
            console.error("Failed to load gateway nodes:", error);
            errorLoadingGwNodes = "Failed to load gateway nodes";
            gwNodes = [];
        } finally {
            loadingGwNodes = false;
        }
    }
    
    // Cleanup
    onMount(() => {
        // Redirect if not logged in
        if (!isLoading && !isLoggedIn) {
            goto('/');
        }
        
        // Load gateway nodes for high-speed mode
        loadGwNodes();
        
        return () => {
            unsubAuthCheck();
        }
    });
    
    // Convert UI Proxy to API format
    function uiToApiProxy(uiProxy: UIProxy, domainConfigs: DomainConfig[]): ExtendedProxy {
        // Generate a temporary ID if needed
        const tempProxyId = uiProxy.id || "temp-" + Date.now();
        
        return {
            id: uiProxy.id,
            title: uiProxy.title,
            addr_listen: uiProxy.listen,
            addr_target: "", // Always send empty string as addr_target
            high_speed: uiProxy.highSpeed || false,
            high_speed_addr: uiProxy.highSpeedAddr || null,
            high_speed_gwid: selectedGwNodeId || null,
            tls_domains: domainConfigs.length > 0 ? domainConfigs.map(config => ({
                sni: config.domain,
                tls: config.useTls,
                tls_pem: config.useTls ? config.certPem || null : null,
                tls_key: config.useTls ? config.certKey || null : null,
                tls_autron: config.useTls ? config.autoTls : false,
                proxy_id: tempProxyId, // Use temp ID since it's new
                gwnode_id: null
            })) : []
        };
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
                addr_target: "", // Server will generate this
                high_speed: proxy.highSpeed,
                high_speed_addr: proxy.highSpeedAddr || null,
                high_speed_gwid: selectedGwNodeId || null
            };
            
            // For new proxies, don't set proxy_id on domains - the backend will handle this
            // Just send the domains with their configuration
            const domainsForApi = domainConfigs.map(config => ({
                ...config,
                // Don't set proxy_id for new proxy - backend will handle it
                proxy_id: "" 
            }));
            
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
</script>

{#if isLoading}
    <LoadingSpinner />
{:else if isLoggedIn}
    <div class="container mx-auto max-w-3xl px-4 py-8">
        <div class="bg-white dark:bg-gray-800 border-1 border-gray-200 dark:border-gray-700 p-6">
            <div class="flex justify-between items-center mb-6">
                <h1 class="text-2xl font-bold text-gray-900 dark:text-white">Add New Proxy</h1>
                <Button variant="secondary" onClick={cancel}>
                    Cancel
                </Button>
            </div>
            
            {#if errorMessage}
                <div class="mb-4 p-3 bg-red-100 dark:bg-red-900/20 border border-red-200 dark:border-red-800 text-red-700 dark:text-red-400 text-sm">
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
                    bind:selectedGwNodeId
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
                                Creating...
                            </span>
                        {:else}
                            Create Proxy
                        {/if}
                    </Button>
                </div>
            </form>
        </div>
    </div>
{/if}