<script lang="ts">
    import { onMount } from 'svelte';
    import { goto } from '$app/navigation';
    import { user } from '$lib/stores/userStore';
    import { Shield, Brain, Globe, Gauge, Lock, FileSearch, Ban, ShieldAlert, FileText, ChevronRight } from 'lucide-svelte';
    import LoadingSpinner from '$lib/components/common/LoadingSpinner.svelte';

    let isLoggedIn = false;
    let isAuthLoading = true;

    const unsubAuth = user.subscribe((value) => {
        isLoggedIn = !!value;
        isAuthLoading = false;
    });

    onMount(() => {
        if (!isAuthLoading && !isLoggedIn) {
            goto('/');
        }
        return () => unsubAuth();
    });

    $: if (!isAuthLoading && !isLoggedIn) {
        goto('/');
    }

    const securityModules = [
        {
            title: 'AI Security',
            description: 'Manage ONNX models for anomaly detection and threat analysis',
            href: '/security/ai',
            icon: Brain,
            disabled: false
        },
        {
            title: 'Country BlockList',
            description: 'Block traffic by country or geographic region',
            href: '/security/country-blocklist',
            icon: Globe,
            disabled: true
        },
        {
            title: 'Rate Limiter',
            description: 'Request rate limiting and throttling controls',
            href: '/security/rate-limiter',
            icon: Gauge,
            disabled: true
        },
        {
            title: 'Port Firewall',
            description: 'Port-based access control and filtering',
            href: '/security/port-firewall',
            icon: Lock,
            disabled: true
        },
        {
            title: 'Payload Sanitizer',
            description: 'Input validation and request sanitization',
            href: '/security/payload-sanitizer',
            icon: FileSearch,
            disabled: true
        },
        {
            title: 'IP Blocklist',
            description: 'Block specific IPs or CIDR ranges',
            href: '/security/ip-blocklist',
            icon: Ban,
            disabled: true
        },
        {
            title: 'WAF Rules',
            description: 'Web Application Firewall pattern matching',
            href: '/security/waf',
            icon: ShieldAlert,
            disabled: true
        },
        {
            title: 'Audit Logs',
            description: 'Security event logging and monitoring',
            href: '/security/audit-logs',
            icon: FileText,
            disabled: true
        }
    ];
</script>

<svelte:head>
    <title>Security</title>
</svelte:head>

<div class="p-6 mx-auto max-w-6xl">
    {#if isAuthLoading}
        <div class="flex justify-center items-center min-h-[400px]">
            <LoadingSpinner />
        </div>
    {:else if isLoggedIn}
        <div class="mb-8">
            <div class="flex items-center gap-3 mb-2">
                <Shield class="h-8 w-8 text-indigo-500" />
                <h1 class="text-2xl font-bold text-gray-900 dark:text-white">Security</h1>
            </div>
            <p class="text-sm text-gray-600 dark:text-gray-400">
                Configure and manage security features for your gateway
            </p>
        </div>

        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-6">
            {#each securityModules as module}
                {#if module.disabled}
                    <div
                        class="block p-6 bg-white dark:bg-[#161b22] border border-gray-200 dark:border-gray-700 rounded-lg opacity-50 cursor-not-allowed relative"
                    >
                        <div class="absolute top-3 right-3">
                            <span class="px-2 py-1 text-xs font-medium bg-gray-200 dark:bg-gray-700 text-gray-600 dark:text-gray-400 rounded">
                                Coming Soon
                            </span>
                        </div>
                        <div class="flex items-start justify-between">
                            <div class="p-2 bg-gray-100 dark:bg-gray-800 rounded-lg">
                                <svelte:component this={module.icon} class="h-6 w-6 text-gray-400 dark:text-gray-500" />
                            </div>
                        </div>
                        <h3 class="mt-4 text-lg font-semibold text-gray-500 dark:text-gray-400">
                            {module.title}
                        </h3>
                        <p class="mt-2 text-sm text-gray-400 dark:text-gray-500">
                            {module.description}
                        </p>
                    </div>
                {:else}
                    <a
                        href={module.href}
                        class="block p-6 bg-white dark:bg-[#161b22] border border-gray-200 dark:border-gray-700 rounded-lg hover:border-indigo-500 dark:hover:border-indigo-500 transition-colors group"
                    >
                        <div class="flex items-start justify-between">
                            <div class="p-2 bg-indigo-100 dark:bg-indigo-900/30 rounded-lg">
                                <svelte:component this={module.icon} class="h-6 w-6 text-indigo-600 dark:text-indigo-400" />
                            </div>
                            <ChevronRight class="h-5 w-5 text-gray-400 group-hover:text-indigo-500 transition-colors" />
                        </div>
                        <h3 class="mt-4 text-lg font-semibold text-gray-900 dark:text-white group-hover:text-indigo-600 dark:group-hover:text-indigo-400 transition-colors">
                            {module.title}
                        </h3>
                        <p class="mt-2 text-sm text-gray-600 dark:text-gray-400">
                            {module.description}
                        </p>
                    </a>
                {/if}
            {/each}
        </div>
    {/if}
</div>
