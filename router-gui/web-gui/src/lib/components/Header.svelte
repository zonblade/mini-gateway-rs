<!-- Header Component -->
<script lang="ts">
    import NavBar from './NavBar.svelte';
    import MobileMenu from './MobileMenu.svelte';
    import { afterUpdate } from 'svelte';
    
    export let username: string;
    export let onLogout: () => void;
    
    let isMobileMenuOpen: boolean = false;
    
    // Toggle mobile menu
    function toggleMobileMenu() {
        isMobileMenuOpen = !isMobileMenuOpen;
        
        // When menu is open, prevent body scrolling
        if (isMobileMenuOpen) {
            document.body.classList.add('overflow-hidden');
        } else {
            document.body.classList.remove('overflow-hidden');
        }
    }
    
    // Close mobile menu
    function closeMobileMenu() {
        if (isMobileMenuOpen) {
            isMobileMenuOpen = false;
            document.body.classList.remove('overflow-hidden');
        }
    }
    
    // Handle click outside to close menu
    function handleClickOutside(event: MouseEvent) {
        const target = event.target as HTMLElement;
        const mobileMenu = document.getElementById('mobile-menu');
        const menuToggle = document.getElementById('menu-toggle');
        
        if (isMobileMenuOpen && mobileMenu && !mobileMenu.contains(target) && 
            menuToggle && !menuToggle.contains(target)) {
            closeMobileMenu();
        }
    }
    
    // Close mobile menu when window is resized to desktop size
    afterUpdate(() => {
        window.addEventListener('resize', () => {
            if (window.innerWidth >= 768 && isMobileMenuOpen) {
                closeMobileMenu();
            }
        });
    });
    
    // Handle click on NavBar
    function handleNavClick(event: MouseEvent) {
        const target = event.target as HTMLElement;
        if (target && target.closest && target.closest('#menu-toggle')) {
            toggleMobileMenu();
        }
    }
</script>

<svelte:window on:click={handleClickOutside} />

<!-- Header/Navigation -->
<header class="bg-white dark:bg-[#161b22] shadow-sm relative z-10">
    <div class="mx-auto px-4 sm:px-6 lg:px-8">
        <NavBar 
            username={username} 
            onLogout={onLogout} 
            on:click={handleNavClick}
        />
    </div>
</header>

<MobileMenu 
    isOpen={isMobileMenuOpen}
    username={username}
    onClose={closeMobileMenu}
    onLogout={onLogout}
/>