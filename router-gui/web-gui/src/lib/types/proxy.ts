export interface Proxy {
    id: string;
    title: string;
    addr_listen: string;
    addr_target: string;
    high_speed: boolean;
    high_speed_addr: string | null;
    tls_domains?: TlsDomain[];
}

export interface TlsDomain {
    id?: string; // Optional for creation
    sni: string;
    tls: boolean;
    tls_pem: string | null;
    tls_key: string | null;
    tls_autron: boolean;
    proxy_id: string; // Set by server
    gwnode_id?: string | null; // Optional gateway node ID
}

export interface ProxyWithDomains {
    proxy: Proxy;
    domains: TlsDomain[];
    warning?: string;
}

export interface ProxyForm {
    id: string;
    title: string;
    addr_listen: string;
    addr_target: string;
    high_speed: boolean;
    high_speed_addr: string;
}

// Local UI model for domain configuration
export interface DomainConfig {
    id: string;
    domain: string;
    useTls: boolean;
    autoTls: boolean;
    certPem: string;
    certKey: string;
    proxy_id: string;
    gwnode_id?: string | null;
}

// Convert API response to form data
export function toFormData(proxy: Proxy): ProxyForm {
    return {
        id: proxy.id || '',
        title: proxy.title || '',
        addr_listen: proxy.addr_listen || '',
        addr_target: proxy.addr_target || '',
        high_speed: proxy.high_speed || false,
        high_speed_addr: proxy.high_speed_addr || ''
    };
}

// Convert form data to API request format
export function toApiData(form: ProxyForm): Proxy {
    return {
        id: form.id || undefined as unknown as string,
        title: form.title,
        addr_listen: form.addr_listen,
        addr_target: form.addr_target,
        high_speed: form.high_speed,
        high_speed_addr: form.high_speed_addr || null
    };
}

// Convert UI domain configs to API TlsDomain format
export function domainsToApiFormat(domains: DomainConfig[]): TlsDomain[] {
    return domains.map(domain => ({
        id: domain.id !== 'new-domain' ? domain.id : undefined,
        sni: domain.domain,
        tls: domain.useTls,
        tls_pem: domain.useTls ? domain.certPem || null : null,
        tls_key: domain.useTls ? domain.certKey || null : null,
        tls_autron: domain.useTls ? domain.autoTls : false,
        proxy_id: domain.proxy_id,
        gwnode_id: domain.gwnode_id || null
    }));
}

// Convert API TlsDomain to UI DomainConfig format
export function apiToDomainConfigs(domains: TlsDomain[]): DomainConfig[] {
    return domains.map((domain, index) => ({
        id: domain.id || `domain-${index}`,
        domain: domain.sni || "",
        useTls: !!domain.tls_pem || !!domain.tls_key || !!domain.tls_autron,
        autoTls: domain.tls_autron || false,
        certPem: domain.tls_pem || "",
        certKey: domain.tls_key || "",
        proxy_id: domain.proxy_id,
        gwnode_id: domain.gwnode_id || null
    }));
}