<script lang="ts">
    import type { Proxy } from "$lib/types/proxy";

    export let proxy: Proxy;
    export let onEdit: () => void;
    export let onDelete: () => void;
</script>

<div
    class="rounded-lg shadow-md overflow-hidden border border-gray-200 dark:border-gray-700 hover:shadow-lg transition-shadow duration-200 min-h-[220px]"
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
                    <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"></path>
                        <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"></path>
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

        <div class="mt-3 overflow-x-auto">
            <table class="min-w-full text-sm">
                <thead class="sr-only">
                    <tr>
                        <th
                            class="text-left font-medium text-gray-600 dark:text-gray-300 pr-2"
                            >Attribute</th
                        >
                        <th class="text-left">Value</th>
                    </tr>
                </thead>
                <tbody class="divide-y divide-gray-200 dark:divide-gray-700">
                    <tr>
                        <td
                            class="py-1 pr-2 font-medium text-gray-600 dark:text-gray-300"
                            >Listen</td
                        >
                        <td class="py-1">:
                            <code
                                class="text-xs bg-gray-100 dark:bg-gray-700 px-1 py-0.5 rounded font-mono"
                            >
                                {proxy.addr_listen}
                            </code>
                        </td>
                    </tr>

                    {#if proxy.tls}
                        <tr>
                            <td
                                class="py-1 pr-2 font-medium text-gray-600 dark:text-gray-300"
                                >TLS</td
                            >
                            <td class="py-1 flex">:&nbsp;
                                <div class="flex items-center">
                                    <span
                                        class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-300"
                                    >
                                        <svg
                                            class="h-3 w-3 mr-1"
                                            fill="currentColor"
                                            viewBox="0 0 20 20"
                                        >
                                            <path
                                                fill-rule="evenodd"
                                                d="M5 9V7a5 5 0 0110 0v2a2 2 0 012 2v5a2 2 0 01-2 2H5a2 2 0 01-2-2v-5a2 2 0 012-2zm8-2v2H7V7a3 3 0 016 0z"
                                                clip-rule="evenodd"
                                            />
                                        </svg>
                                        {#if proxy.tls_autron}
                                            Auto Renew
                                        {:else}
                                            Secured
                                        {/if}
                                    </span>
                                </div>
                            </td>
                        </tr>
                    {:else}
                        <tr>
                            <td
                                class="py-1 pr-2 font-medium text-gray-600 dark:text-gray-300"
                                >TLS</td
                            >
                            <td class="py-1">:
                                <span class="text-gray-500 dark:text-gray-400"
                                    >Disabled</span
                                >
                            </td>
                        </tr>
                    {/if}

                    <tr>
                        <td
                            class="py-1 pr-2 font-medium text-gray-600 dark:text-gray-300"
                            >Domain</td
                        >
                        <td class="py-1">:
                            {#if proxy.sni}
                                <code
                                    class="text-xs bg-gray-100 dark:bg-gray-700 px-1 py-0.5 rounded font-mono"
                                >
                                    {proxy.sni}
                                </code>
                            {:else}
                                <span class="text-gray-500 dark:text-gray-400"
                                    >-</span
                                >
                            {/if}
                        </td>
                    </tr>

                    <tr>
                        <td
                            class="py-1 pr-2 font-medium text-gray-600 dark:text-gray-300"
                            >Mode</td
                        >
                        <td class="py-1 flex">:&nbsp;
                            {#if proxy.high_speed}
                                <div class="flex items-center">
                                    <span
                                        class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-purple-100 text-purple-800 dark:bg-purple-900 dark:text-purple-300"
                                    >
                                        <svg
                                            xmlns="http://www.w3.org/2000/svg"
                                            class="h-3 w-3 mr-1"
                                            viewBox="0 0 20 20"
                                            fill="currentColor"
                                        >
                                            <path
                                                fill-rule="evenodd"
                                                d="M11.3 1.046A1 1 0 0112 2v5h4a1 1 0 01.82 1.573l-7 10A1 1 0 018 18v-5H4a1 1 0 01-.82-1.573l7-10a1 1 0 011.12-.38z"
                                                clip-rule="evenodd"
                                            />
                                        </svg>
                                        High-Speed Mode
                                    </span>
                                </div>
                            {:else}
                                <div class="flex items-center">
                                    <span
                                        class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-300"
                                    >
                                        <svg
                                            xmlns="http://www.w3.org/2000/svg"
                                            class="h-3 w-3 mr-1"
                                            viewBox="0 0 20 20"
                                            fill="currentColor"
                                        >
                                            <path
                                                fill-rule="evenodd"
                                                d="M3 4a1 1 0 011-1h12a1 1 0 011 1v12a1 1 0 01-1 1H4a1 1 0 01-1-1V4zm2 1v10h8V5H5zm4 4a1 1 0 100 2 1 1 0 000-2z"
                                                clip-rule="evenodd"
                                            />
                                            <path
                                                d="M10.707 8.707a1 1 0 00-1.414 0L8 10.086V12h1.914l1.293-1.293a1 1 0 000-1.414l-.5-.5z"
                                            />
                                        </svg>
                                        Gateway Node Mode
                                    </span>
                                </div>
                            {/if}
                        </td>
                    </tr>
                    <tr>
                        <td
                            class="py-1 pr-2 font-medium text-gray-600 dark:text-gray-300"
                        >Binding</td>
                        <td class="py-1">:
                            {#if proxy.high_speed}
                                {#if proxy.high_speed_addr}
                                    <code
                                        class="text-xs bg-gray-100 dark:bg-gray-700 px-1 py-0.5 rounded font-mono"
                                    >
                                        {proxy.high_speed_addr}
                                    </code>
                                {:else}
                                    <span
                                        class="text-gray-500 dark:text-gray-400"
                                        >High Speed Enabled</span
                                    >
                                {/if}
                            {:else}
                                <span class="text-gray-500 dark:text-gray-400"
                                >Multiple Listener</span>
                            {/if}
                        </td>
                    </tr>
                </tbody>
            </table>
        </div>
    </div>
</div>
