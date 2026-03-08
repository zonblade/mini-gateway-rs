<script lang="ts">
    export let data: number[] = [];
    export let color: string = '#3b82f6';
    export let height: number = 40;
    export let strokeWidth: number = 2;

    $: points = generatePath(data);

    function generatePath(values: number[]): string {
        if (values.length === 0) return '';

        const max = Math.max(...values, 1);
        const min = Math.min(...values, 0);
        const range = max - min || 1;

        const width = 100; // viewBox width
        const padding = 2;
        const effectiveHeight = height - padding * 2;
        const effectiveWidth = width - padding * 2;

        const stepX = values.length > 1 ? effectiveWidth / (values.length - 1) : 0;

        return values
            .map((v, i) => {
                const x = padding + i * stepX;
                const y = padding + effectiveHeight - ((v - min) / range) * effectiveHeight;
                return `${i === 0 ? 'M' : 'L'} ${x.toFixed(1)} ${y.toFixed(1)}`;
            })
            .join(' ');
    }
</script>

<svg
    viewBox="0 0 100 {height}"
    class="w-full h-full"
    preserveAspectRatio="none"
>
    {#if points}
        <path
            d={points}
            fill="none"
            stroke={color}
            stroke-width={strokeWidth}
            stroke-linecap="round"
            stroke-linejoin="round"
            vector-effect="non-scaling-stroke"
        />
    {:else}
        <line
            x1="2" y1={height / 2}
            x2="98" y2={height / 2}
            stroke="#e5e7eb"
            stroke-width="1"
            stroke-dasharray="4 2"
        />
    {/if}
</svg>
