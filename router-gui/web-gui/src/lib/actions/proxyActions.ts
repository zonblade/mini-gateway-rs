import type { Proxy, ProxyWithDomains, TlsDomain, DomainConfig } from '$lib/types/proxy';
import { proxyService } from '$lib/services/proxyService';
import { domainsToApiFormat } from '$lib/types/proxy';

// Proxy actions to be used by components
export const proxyActions = {
    // Fetch all proxies with their domains
    getProxies: async (): Promise<ProxyWithDomains[]|null> => {
        try {
            return await proxyService.getAllProxies();
        } catch (error) {
            console.error('Failed to fetch proxies:', error);
            return null; // Return null on error
        }
    },
    
    // Get a single proxy by ID with its domains
    getProxyById: async (proxyId: string): Promise<ProxyWithDomains> => {
        try {
            return await proxyService.getProxyById(proxyId);
        } catch (error) {
            console.error(`Failed to fetch proxy ${proxyId}:`, error);
            throw error;
        }
    },
    
    // Create or update a proxy with domains
    saveProxy: async (proxy: Proxy, domains?: DomainConfig[]): Promise<ProxyWithDomains> => {
        try {
            // Convert domain configs to the API format
            let apiDomains: TlsDomain[] = [];
            
            if (domains && domains.length > 0) {
                apiDomains = domains.map(domain => {
                    // Create new domain object with correct fields
                    const apiDomain: any = {
                        // Use empty id for new domains so server will generate one
                        id: domain.id && !domain.id.startsWith('domain-') ? domain.id : undefined,
                        // Map domain name to sni field
                        sni: domain.domain,
                        // Include tls flag
                        tls: domain.useTls,
                        // TLS cert data if TLS is enabled
                        tls_pem: domain.useTls ? domain.certPem || null : null,
                        tls_key: domain.useTls ? domain.certKey || null : null,
                        tls_autron: domain.useTls ? domain.autoTls : false,
                        // Use gateway node ID if provided
                        gwnode_id: domain.gwnode_id || null
                    };
                    
                    // Only include proxy_id for existing proxies and existing domains
                    // For new proxies, the server will assign the right proxy_id
                    if (proxy.id && proxy.id.length > 0 && !proxy.id.startsWith('temp-')) {
                        apiDomain.proxy_id = domain.proxy_id || proxy.id;
                    }
                    
                    return apiDomain;
                });
            }

            return await proxyService.saveProxy(proxy, apiDomains);
        } catch (error: any) {
            console.error('Error saving proxy:', error);
            throw error.error??error;
        }
    },
    
    // Delete a proxy
    deleteProxy: async (proxyId: string): Promise<boolean> => {
        try {
            await proxyService.deleteProxy(proxyId);
            return true;
        } catch (error) {
            console.error(`Failed to delete proxy ${proxyId}:`, error);
            throw error;
        }
    },
    
    // Sync proxies with server nodes
    syncProxies: async (): Promise<{ status: string, message: string }> => {
        try {
            return await proxyService.syncProxyNodes();
        } catch (error) {
            console.error('Failed to sync proxy nodes:', error);
            throw error;
        }
    }
};

export default proxyActions;