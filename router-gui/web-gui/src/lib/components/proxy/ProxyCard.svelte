<script lang="ts">
    import type { Proxy, TlsDomain } from "$lib/types/proxy";

    export let proxy: Proxy;
    export let domains: TlsDomain[] = [];
    export let onEdit: () => void;
    export let onDelete: () => void;

    // Get the main domain or the first domain in the list
    $: mainDomain = domains.length > 0 ? domains[0].sni : null;

    // Calculate TLS stats
    $: tlsEnabledCount = domains.filter((d) => d.tls).length;
    $: tlsDisabledCount = domains.length - tlsEnabledCount;
</script>

<div
    class="overflow-hidden border border-gray-200 dark:border-gray-700 hover:bg-white/40 hover:dark:bg-gray-800/40 hover:border-gray-300 dark:hover:border-gray-600 duration-200 min-h-[220px]"
>
    <div class="p-4">
        <div class="flex justify-between items-start">
            <h3
                class="text-lg font-medium text-gray-900 dark:text-white truncate"
            >
                {proxy.title}
            </h3>
            <div class="flex space-x-1">
                <button
                    on:click|preventDefault={onEdit}
                    class="text-blue-500 hover:text-blue-700 dark:text-blue-400 dark:hover:text-blue-300 p-1"
                    aria-label="Edit proxy"
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
                    on:click={onDelete}
                    class="text-red-500 hover:text-red-700 dark:text-red-400 dark:hover:text-red-300 p-1"
                    aria-label="Delete proxy"
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
                            d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"
                        ></path>
                        <line x1="10" y1="11" x2="10" y2="17"></line>
                        <line x1="14" y1="11" x2="14" y2="17"></line>
                    </svg>
                </button>
            </div>
        </div>
        <div class="mt-2 text-sm text-gray-600 dark:text-gray-400">
            <table class="w-full">
                <tbody>
                    <tr>
                        <td class="py-1 font-medium w-1/3">Listen</td>
                        <td class="py-1 truncate">:&ensp;{proxy.addr_listen}</td
                        >
                    </tr>
                    <tr>
                        <td class="py-1 font-medium w-1/3"
                            >Domain{domains.length > 1 ? "s" : ""}</td
                        >
                        <td class="py-1">
                            {#if domains.length > 0}
                                <span class="truncate text-gray-500"
                                    >:&ensp;{domains.length} Domain{domains.length >
                                    1
                                        ? "s"
                                        : ""}</span
                                >
                            {:else}
                                <span class="text-gray-500"
                                    >:&ensp;No domains</span
                                >
                            {/if}
                        </td>
                    </tr>
                    <tr>
                        <td class="py-1 font-medium w-1/3">Node Type</td>
                        <td class="py-1 flex">
                            :&ensp;
                            {#if proxy.high_speed}
                                <span
                                    class="text-purple-600 dark:text-purple-400 font-medium flex items-center"
                                >
                                    <svg
                                        xmlns="http://www.w3.org/2000/svg"
                                        class="h-4 w-4 mr-1"
                                        viewBox="0 0 20 20"
                                        fill="currentColor"
                                    >
                                        <path
                                            fill-rule="evenodd"
                                            d="M11.3 1.046A1 1 0 0112 2v5h4a1 1 0 01.82 1.573l-7 10A1 1 0 018 18v-5H4a1 1 0 01-.82-1.573l7-10a1 1 0 011.12-.38z"
                                            clip-rule="evenodd"
                                        />
                                    </svg>
                                    High Speed
                                </span>
                            {:else}
                                <span
                                    class="text-blue-400 dark:text-blue-300 flex items-center"
                                >
                                    <svg
                                        xmlns="http://www.w3.org/2000/svg"
                                        class="h-4 w-4 mr-1"
                                        viewBox="0 0 20 20"
                                        fill="currentColor"
                                    >
                                        <path
                                            d="M5.5 16a3.5 3.5 0 01-.369-6.98 4 4 0 117.753-1.977A4.5 4.5 0 1113.5 16h-8z"
                                        />
                                    </svg>
                                    Multiple
                                </span>
                            {/if}
                        </td>
                    </tr>
                    <tr>
                        <td class="py-1 font-medium w-1/3">TLS Status</td>
                        <td class="py-1 flex">
                            :&ensp;
                            {#if tlsEnabledCount > 0}
                                <span
                                    class="text-emerald-500 dark:text-emerald-400 font-medium flex items-center"
                                >
                                    <svg
                                        xmlns="http://www.w3.org/2000/svg"
                                        class="h-4 w-4 mr-1"
                                        viewBox="0 0 20 20"
                                        fill="currentColor"
                                    >
                                        <path
                                            fill-rule="evenodd"
                                            d="M5 9V7a5 5 0 0110 0v2a2 2 0 012 2v5a2 2 0 01-2 2H5a2 2 0 01-2-2v-5a2 2 0 012-2zm8-2v2H7V7a3 3 0 016 0z"
                                            clip-rule="evenodd"
                                        />
                                    </svg>
                                    Enabled
                                </span>
                            {:else}
                                <span
                                    class="text-gray-400 dark:text-gray-500 flex items-center"
                                >
                                    <svg
                                        xmlns="http://www.w3.org/2000/svg"
                                        class="h-4 w-4 mr-1"
                                        viewBox="0 0 20 20"
                                        fill="currentColor"
                                    >
                                        <path
                                            d="M10 2a5 5 0 00-5 5v2a2 2 0 00-2 2v5a2 2 0 002 2h10a2 2 0 002-2v-5a2 2 0 00-2-2H7V7a3 3 0 015.905-.75 1 1 0 001.937-.5A5.002 5.002 0 0010 2z"
                                        />
                                    </svg>
                                    Disabled
                                </span>
                            {/if}
                        </td>
                    </tr>
                    <tr>
                        <td class="py-1 text-sm italic w-1/3 pl-[20px]">TLS</td>
                        <td class="py-1">:&ensp;<span class="text-gray-500">{tlsEnabledCount} Domain{tlsEnabledCount>1 ? 's' : ''}</span> </td>
                    </tr>
                    <tr>
                        <td class="py-1 text-sm italic w-1/3 pl-[20px]">TCP</td>
                        <td class="py-1"
                            >:&ensp;<span class="text-gray-500">{tlsDisabledCount} Domain{tlsDisabledCount>1 ? 's' : ''}</span> {#if tlsEnabledCount > 0 && tlsDisabledCount > 0}<span
                                    class="text-red-600 dark:text-red-400 font-semibold"
                                    >(ignored)</span
                                >{/if}
                        </td>
                    </tr>
                </tbody>
            </table>

            <div class="mt-2">
                {#if proxy.high_speed}
                    <!-- Warning with more vibrant pastel style -->
                    <div
                        class="bg-pink-100 dark:bg-pink-900/40 text-pink-800 dark:text-pink-200 text-xs mb-2 p-2 rounded-md flex items-center border border-pink-200 dark:border-pink-800"
                    >
                        <svg
                            xmlns="http://www.w3.org/2000/svg"
                            class="h-6 min-w-[32px] mr-1"
                            fill="none"
                            viewBox="0 0 24 24"
                            stroke="currentColor"
                        >
                            <path
                                stroke-linecap="round"
                                stroke-linejoin="round"
                                stroke-width="2"
                                d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
                            />
                        </svg>
                        <span
                            >If High speed enabled, only selected Gateway domain
                            will be enabled.</span
                        >
                    </div>
                {/if}
                {#if tlsEnabledCount > 0 && tlsDisabledCount > 0}
                    <!-- Warning with more vibrant pastel style -->
                    <div
                        class="bg-orange-100 dark:bg-orange-900/40 text-orange-800 dark:text-orange-200 text-xs mb-2 p-2 rounded-md flex items-center border border-orange-200 dark:border-orange-800"
                    >
                        <svg
                            xmlns="http://www.w3.org/2000/svg"
                            class="h-6 min-w-[32px] mr-1"
                            fill="none"
                            viewBox="0 0 24 24"
                            stroke="currentColor"
                        >
                            <path
                                stroke-linecap="round"
                                stroke-linejoin="round"
                                stroke-width="2"
                                d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
                            />
                        </svg>
                        <span
                            >If one of the domain is enabling TLS, the non-TLS
                            domain will be ignored.</span
                        >
                    </div>
                {/if}
            </div>
        </div>
    </div>
</div>
