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

<div class="flex">
    <!-- Navigation Sidebar -->
    <NavBar 
        username={username} 
        onLogout={onLogout} 
        on:click={handleNavClick}
    />
    
    <!-- Main Content Area with padding for sidebar -->
    <div class="md:ml-64 w-full transition-all duration-300 ease-in-out">
        <!-- Mobile Header appears here, but is styled in NavBar component -->
        <div class="md:hidden">
            <!-- This div is intentionally empty as mobile header is in NavBar component -->
        </div>
        
        <MobileMenu 
            isOpen={isMobileMenuOpen}
            username={username}
            onClose={closeMobileMenu}
            onLogout={onLogout}
        />
    </div>
</div>