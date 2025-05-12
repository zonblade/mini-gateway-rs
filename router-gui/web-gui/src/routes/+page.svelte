<script lang="ts">
    import { onMount } from "svelte";
    import { user } from "$lib/stores/userStore";
    import Login from "$lib/components/Login.svelte";
    import Dashboard from "$lib/components/Dashboard.svelte";
    import LineChart from "$lib/components/LineChart.svelte";
    import HighLowChart from "$lib/components/HighLowChart.svelte";

    // Authentication state
    let isLoggedIn = false;

    // Subscribe to the user store
    const unsubscribe = user.subscribe((value) => {
        isLoggedIn = !!value;
    });

    // Clean up subscription when component is destroyed
    onMount(() => {
        return unsubscribe;
    });

    // Handle successful login
    function handleLoginSuccess() {
        // Update the state (the user store subscription will handle this)
    }

    // // Generate random high-low data with proper relationships
    // function generateRandomHighLowData() {
    //     // Number of data points
    //     const numPoints = 120;

    //     // Generate random data with constraints (high > avg > low)
    //     const generateSeries = (baseMin, baseMax, name, color, avgColor) => {
    //         const lows = [];
    //         const highs = [];
    //         const avgs = [];

    //         for (let i = 0; i < numPoints; i++) {
    //             // Generate base values with randomness
    //             const baseValue = Math.random() * (baseMax - baseMin) + baseMin;
    //             const range = baseValue * 0.3; // 30% fluctuation for range

    //             // Ensure high > avg > low
    //             const low = baseValue - Math.random() * range;
    //             const high = baseValue + Math.random() * range;
    //             const avg = low + Math.random() * (high - low);

    //             lows.push(Math.round(low));
    //             highs.push(Math.round(high));
    //             avgs.push(Math.round(avg));
    //         }

    //         return {
    //             name,
    //             color,
    //             avgColor,
    //             highs,
    //             lows,
    //             avgs,
    //         };
    //     };

    //     return [generateSeries(10, 20, "Humidity", "#2ca02c", "#d62728")];
    // }

    // const highLowData = generateRandomHighLowData();
    // const labels = ["Mon", "Tue"];

    // const Ldata = [
    //     {
    //         label: "Crypto Signal",
    //         values: generateCryptoLikeData(120),
    //         color: "#ff6384",
    //     },
    // ];

    // // Function to generate crypto-like price movements
    // function generateCryptoLikeData(length, options = {}) {
    //     // Default parameters
    //     const {
    //         initialValue = 100,
    //         volatility = 3.15,
    //         trendStrength = 0.001,
    //         trendChangeProbability = 0.03,
    //         volatilityClusterStrength = 0.85,
    //         spikeProbability = 0.01,
    //         maxSpikeMultiplier = 0.08,
    //     } = options;

    //     const values = [];
    //     let currentValue = initialValue;
    //     let currentVolatility = volatility;
    //     let currentTrend = 0; // No initial trend

    //     for (let i = 0; i < length; i++) {
    //         // Randomly change trend direction
    //         if (Math.random() < trendChangeProbability) {
    //             currentTrend = (Math.random() - 0.48) * trendStrength * 5; // Slight bullish bias
    //         }

    //         // Generate random price movement
    //         let randomWalk =
    //             (Math.random() - 0.5) * currentVolatility * currentValue;

    //         // Add trend component
    //         let trendComponent = currentValue * currentTrend;

    //         // Occasional spikes (both up and down)
    //         let spikeComponent = 0;
    //         if (Math.random() < spikeProbability) {
    //             spikeComponent =
    //                 currentValue * (Math.random() - 0.5) * maxSpikeMultiplier;
    //         }

    //         // Calculate new value
    //         currentValue = Math.max(
    //             1,
    //             currentValue + randomWalk + trendComponent + spikeComponent,
    //         );

    //         // Volatility clustering (volatility tends to persist)
    //         currentVolatility = Math.max(
    //             0.005,
    //             volatility * (1 - volatilityClusterStrength) +
    //                 currentVolatility * volatilityClusterStrength +
    //                 (Math.random() - 0.5) * 0.005,
    //         );

    //         values.push(Math.round(currentValue));
    //     }

    //     return values;
    // }
</script>

{#if isLoggedIn}
    <Dashboard />
{:else}
    <Login onLoginSuccess={handleLoginSuccess} />
    <!-- <div class="min-h-[300px] bg-cyan-300">
        <div style="width: 100%; height: 300px;">
            <LineChart data={Ldata} {labels} />
        </div>
    </div>
    <div class="min-h-[300px]">
        <div style="width: 100%; height: 300px;">
            <HighLowChart data={highLowData} {labels} />
        </div>
    </div> -->
{/if}

<style lang="postcss">
    @import "tailwindcss";

    :global(html) {
        @apply transition-colors duration-300;
    }

    :global(html.dark) {
        color-scheme: dark;
    }

    /* Prevent flashing of unstyled content by hiding the body until theme is applied */
    :global(body) {
        visibility: visible;
    }

    :global(body.loading) {
        visibility: hidden;
    }
</style>
