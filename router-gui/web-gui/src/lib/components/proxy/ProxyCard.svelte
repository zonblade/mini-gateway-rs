<script lang="ts">
    import type { Proxy, TlsDomain } from "$lib/types/proxy";
    import { Edit, Trash2 } from "lucide-svelte";
    import Button from "../common/Button.svelte";

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
    class="overflow-hidden hover:shadow-md border border-gray-200 dark:border-gray-700 hover:bg-white/40 hover:dark:bg-gray-800/40 hover:border-gray-300 dark:hover:border-gray-600 duration-200 min-h-[220px]"
>
    <div>
        <div class="flex justify-end items-start px-4 py-2">
            <div class="flex space-x-2">
                <Button variant="secondary" size="sm" onClick={onEdit}>
                    <Edit size={16} />
                    Modify
                </Button>
                <Button variant="danger" size="sm" onClick={onDelete}>
                    <Trash2 size={16} />
                    Delete
                </Button>
            </div>
        </div>
        <div class="text-sm text-gray-600 dark:text-gray-400 pb-4 px-4">
            <h3
                class="text-lg font-medium text-gray-900 dark:text-white truncate"
            >
                {proxy.title}
            </h3>
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
                        <td class="py-1"
                            >:&ensp;<span class="text-gray-500"
                                >{tlsEnabledCount} Domain{tlsEnabledCount > 1
                                    ? "s"
                                    : ""}</span
                            >
                        </td>
                    </tr>
                    <tr>
                        <td class="py-1 text-sm italic w-1/3 pl-[20px]">TCP</td>
                        <td class="py-1"
                            >:&ensp;<span class="text-gray-500"
                                >{tlsDisabledCount} Domain{tlsDisabledCount > 1
                                    ? "s"
                                    : ""}</span
                            >
                            {#if tlsEnabledCount > 0 && tlsDisabledCount > 0}<span
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
                        class="bg-pink-100 dark:bg-pink-900/40 text-pink-800 dark:text-pink-200 text-xs mb-2 p-2 flex items-center border border-pink-200 dark:border-pink-800"
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
                        class="bg-orange-100 dark:bg-orange-900/40 text-orange-800 dark:text-orange-200 text-xs mb-2 p-2 flex items-center border border-orange-200 dark:border-orange-800"
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
