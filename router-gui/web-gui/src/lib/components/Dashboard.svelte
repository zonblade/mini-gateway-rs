<script lang="ts">
    import { user } from "$lib/stores/userStore";

    let username = "";

    // Subscribe to the user store to get the current username
    const unsubscribe = user.subscribe((value) => {
        if (value) {
            username = value.username;
        }
    });

    // Clean up subscription when component is destroyed
    import { onDestroy } from "svelte";
    import LineChart from "./LineChart.svelte";
    import HighLowChart from "./HighLowChart.svelte";

    // Define interfaces for high-low data
    interface HighLowSeries {
        name: string;
        color: string;
        avgColor: string;
        highs: number[];
        lows: number[];
        avgs: number[];
    }

    // Generate random high-low data with proper relationships
    function generateRandomHighLowData(): HighLowSeries[] {
        // Number of data points
        const numPoints = 120;

        // Generate random data with constraints (high > avg > low)
        const generateSeries = (baseMin: number, baseMax: number, name: string, color: string, avgColor: string) => {
            const lows = [];
            const highs = [];
            const avgs = [];

            for (let i = 0; i < numPoints; i++) {
                // Generate base values with randomness
                const baseValue = Math.random() * (baseMax - baseMin) + baseMin;
                const range = baseValue * 0.3; // 30% fluctuation for range

                // Ensure high > avg > low
                const low = baseValue - Math.random() * range;
                const high = baseValue + Math.random() * range;
                const avg = low + Math.random() * (high - low);

                lows.push(Math.round(low));
                highs.push(Math.round(high));
                avgs.push(Math.round(avg));
            }

            return {
                name,
                color,
                avgColor,
                highs,
                lows,
                avgs,
            };
        };

        return [generateSeries(10, 20, "Humidity", "#2ca02c", "#d62728")];
    }

    const highLowData: HighLowSeries[] = generateRandomHighLowData();
    const labels: string[] = ["Mon", "Tue"];

    interface LineDataSeries {
        label: string;
        values: number[];
        color: string;
    }

    const Ldata: LineDataSeries[] = [
        {
            label: "Crypto Signal",
            values: generateCryptoLikeData(120),
            color: "#ff6384",
        },
    ];

    // Function to generate crypto-like price movements
    interface CryptoDataOptions {
        initialValue?: number;
        volatility?: number;
        trendStrength?: number;
        trendChangeProbability?: number;
        volatilityClusterStrength?: number;
        spikeProbability?: number;
        maxSpikeMultiplier?: number;
    }

    function generateCryptoLikeData(length: number, options: CryptoDataOptions = {}) {
        // Default parameters
        const {
            initialValue = 100,
            volatility = 3.15,
            trendStrength = 0.001,
            trendChangeProbability = 0.03,
            volatilityClusterStrength = 0.85,
            spikeProbability = 0.01,
            maxSpikeMultiplier = 0.08,
        } = options;

        const values = [];
        let currentValue = initialValue;
        let currentVolatility = volatility;
        let currentTrend = 0; // No initial trend

        for (let i = 0; i < length; i++) {
            // Randomly change trend direction
            if (Math.random() < trendChangeProbability) {
                currentTrend = (Math.random() - 0.48) * trendStrength * 5; // Slight bullish bias
            }

            // Generate random price movement
            let randomWalk =
                (Math.random() - 0.5) * currentVolatility * currentValue;

            // Add trend component
            let trendComponent = currentValue * currentTrend;

            // Occasional spikes (both up and down)
            let spikeComponent = 0;
            if (Math.random() < spikeProbability) {
                spikeComponent =
                    currentValue * (Math.random() - 0.5) * maxSpikeMultiplier;
            }

            // Calculate new value
            currentValue = Math.max(
                1,
                currentValue + randomWalk + trendComponent + spikeComponent,
            );

            // Volatility clustering (volatility tends to persist)
            currentVolatility = Math.max(
                0.005,
                volatility * (1 - volatilityClusterStrength) +
                    currentVolatility * volatilityClusterStrength +
                    (Math.random() - 0.5) * 0.005,
            );

            values.push(Math.round(currentValue));
        }

        return values;
    }

    onDestroy(unsubscribe);
</script>

<div class="p-6 max-w-4xl mx-auto">
    <div class="bg-white dark:bg-[#161b22] shadow-sm rounded-lg p-6">
        <h1 class="text-3xl font-bold mb-6">Hello World!</h1>

        <div
            class="mb-6 p-4 bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-md"
        >
            <p class="text-blue-800 dark:text-blue-300">
                Welcome to Mini Gateway, <span class="font-medium"
                    >{username}</span
                >! You have successfully logged in.
            </p>
        </div>

        <div class="space-y-4">
            <h2 class="text-xl font-semibold">Your Gateway Details</h2>

            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div class="p-4 bg-gray-50 dark:bg-gray-800/50 rounded-md">
                    <h3 class="font-medium mb-2">Token</h3>
                    <p
                        class="text-sm text-gray-500 dark:text-gray-400 break-all"
                    >
                        Secured token is stored in the browser
                    </p>
                </div>

                <div class="p-4 bg-gray-50 dark:bg-gray-800/50 rounded-md">
                    <h3 class="font-medium mb-2">Session</h3>
                    <p class="text-sm text-gray-500 dark:text-gray-400">
                        Active until you log out
                    </p>
                </div>
            </div>
        </div>
    </div>
    <div class="mt-6 p-4 bg-gray-50 dark:bg-gray-800/50 rounded-md">
        <div class="min-h-[300px] ">
            <div style="width: 100%; height: 300px;">
                <LineChart data={Ldata} {labels} />
            </div>
        </div>
        <div class="min-h-[300px]">
            <div style="width: 100%; height: 300px;">
                <HighLowChart data={highLowData} {labels} />
            </div>
        </div>
    </div>
</div>
