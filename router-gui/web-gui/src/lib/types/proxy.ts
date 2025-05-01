export interface Proxy {
    id: string;
    title: string;
    addr_listen: string;
    addr_target: string;
    tls: boolean;
    tls_pem: string | null;
    tls_key: string | null;
    tls_autron: boolean;
    sni: string | null;
    high_speed: boolean;
    high_speed_addr: string | null;
}

export interface ProxyForm {
    id: string;
    title: string;
    addr_listen: string;
    addr_target: string;
    tls: boolean;
    tls_pem: string;
    tls_key: string;
    tls_autron: boolean;
    sni: string;
    high_speed: boolean;
    high_speed_addr: string;
}

// Convert API response to form data
export function toFormData(proxy: Proxy): ProxyForm {
    return {
        id: proxy.id || '',
        title: proxy.title || '',
        addr_listen: proxy.addr_listen || '',
        addr_target: proxy.addr_target || '',
        tls: proxy.tls || false,
        tls_pem: proxy.tls_pem || '',
        tls_key: proxy.tls_key || '',
        tls_autron: proxy.tls_autron || false,
        sni: proxy.sni || '',
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
        tls: form.tls,
        tls_pem: form.tls_pem || null,
        tls_key: form.tls_key || null,
        tls_autron: form.tls_autron,
        sni: form.sni || null,
        high_speed: form.high_speed,
        high_speed_addr: form.high_speed_addr || null
    };
}