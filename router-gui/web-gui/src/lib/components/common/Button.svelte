<script lang="ts">
    export let type: "button" | "submit" | "reset" = "button";
    export let variant: "primary" | "secondary" | "danger" | "ghost" = "primary";
    export let size: "sm" | "md" | "lg" = "md";
    export let disabled = false;
    export let fullWidth = false;
    export let onClick: () => void = () => {};
    
    // Compute classes based on props
    $: variantClasses = {
        primary: "bg-blue-600 hover:bg-blue-700 text-white border-transparent outline-blue-300 outline-1 dark:outline-none",
        secondary: "bg-white hover:bg-gray-50 text-gray-700 outline-gray-300 outline-1 dark:outline-none",
        danger: "bg-red-600 hover:bg-red-700 text-white border-transparent outline-red-300 outline-1 dark:outline-none",
        ghost: "bg-transparent hover:bg-gray-100 dark:hover:bg-gray-800 text-gray-700 dark:text-gray-300 outline-gray-300 outline-1 dark:outline-none"
    }[variant];
    
    $: sizeClasses = {
        sm: "px-3 py-1 text-xs",
        md: "px-4 py-2 text-sm",
        lg: "px-5 py-2.5 text-base"
    }[size];
    
    $: widthClass = fullWidth ? "w-full" : "";
    
    $: disabledClasses = disabled ? "opacity-50 cursor-not-allowed" : "";
</script>

<button
    {type}
    class="flex gap-1 {variantClasses} {sizeClasses} {widthClass} {disabledClasses} font-medium focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 transition-colors"
    on:click={onClick}
    {disabled}
>
    <slot />
</button>