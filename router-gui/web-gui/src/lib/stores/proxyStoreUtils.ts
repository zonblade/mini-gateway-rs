import { derived } from 'svelte/store';
import { proxyStore } from './proxyStore';

// Create a derived store for filtered and paginated proxies
export const filteredProxies = derived(
    proxyStore,
    $proxyStore => {
        return proxyStore.getFilteredProxies($proxyStore);
    }
);

// Create a derived store that indicates if we should show the "Load More" button
export const canLoadMore = derived(
    proxyStore,
    $proxyStore => $proxyStore.hasMore && !$proxyStore.loading
);

// Create a derived store for the loading state
export const isLoading = derived(
    proxyStore,
    $proxyStore => $proxyStore.loading
);

// Create a derived store for the error state
export const proxyError = derived(
    proxyStore,
    $proxyStore => $proxyStore.error
);