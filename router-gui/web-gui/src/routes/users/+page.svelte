<script lang="ts">
    import { onMount } from "svelte";
    import { goto } from "$app/navigation";
    import { user } from "$lib/stores/userStore";
    import SearchBar from "$lib/components/users/SearchBar.svelte";
    import UsersTable from "$lib/components/users/UsersTable.svelte";
    import Pagination from "$lib/components/users/Pagination.svelte";
    import UserModal from "$lib/components/users/UserModal.svelte";
    
    // Define User interface for type safety
    interface User {
        id: number;
        username: string;
        email: string;
        role: string;
        active: boolean;
    }
    
    // Authentication and loading states
    let isLoggedIn = false;
    let isLoading = true; // Add loading state
    
    const unsubAuthCheck = user.subscribe(value => {
        isLoggedIn = !!value;
        isLoading = false; // Set loading to false once we've checked auth status
    });
    
    onMount(() => {
        // Redirect happens after auth check is complete
        if (!isLoading && !isLoggedIn) {
            goto('/');
        }
        
        return () => {
            unsubAuthCheck(); // Clean up subscription
        };
    });
    
    // Mock user data for demonstration
    let users: User[] = [
        { id: 1, username: "admin", email: "admin@example.com", role: "Admin", active: true },
        { id: 2, username: "user1", email: "user1@example.com", role: "User", active: true },
        { id: 3, username: "user2", email: "user2@example.com", role: "User", active: false },
        { id: 4, username: "support", email: "support@example.com", role: "Support", active: true },
        { id: 5, username: "guest", email: "guest@example.com", role: "Guest", active: true },
        // Add more mock users for pagination testing
        { id: 6, username: "user6", email: "user6@example.com", role: "User", active: true },
        { id: 7, username: "user7", email: "user7@example.com", role: "User", active: true },
        { id: 8, username: "user8", email: "user8@example.com", role: "User", active: false },
        { id: 9, username: "user9", email: "user9@example.com", role: "User", active: true },
        { id: 10, username: "user10", email: "user10@example.com", role: "User", active: true },
        { id: 11, username: "user11", email: "user11@example.com", role: "User", active: true },
        { id: 12, username: "user12", email: "user12@example.com", role: "User", active: false },
    ];

    // For add/edit user popup
    let showUserModal = false;
    let isEditMode = false;
    let currentUser: User = { id: 0, username: "", email: "", role: "User", active: true };
    
    // Search functionality
    let searchTerm = "";
    $: filteredUsers = users.filter(user => 
        user.username.toLowerCase().includes(searchTerm.toLowerCase()) ||
        user.email.toLowerCase().includes(searchTerm.toLowerCase()) ||
        user.role.toLowerCase().includes(searchTerm.toLowerCase())
    );
    
    // Pagination
    let currentPage = 1;
    let itemsPerPage = 5;
    $: totalPages = Math.ceil(filteredUsers.length / itemsPerPage);
    $: paginatedUsers = filteredUsers.slice(
        (currentPage - 1) * itemsPerPage,
        currentPage * itemsPerPage
    );
    
    // Reset to first page when search term changes
    $: if (searchTerm) {
        currentPage = 1;
    }
    
    // Function to open modal for adding a new user
    function addUser(): void {
        currentUser = { id: 0, username: "", email: "", role: "User", active: true };
        isEditMode = false;
        showUserModal = true;
    }
    
    // Function to open modal for editing an existing user
    function editUser(user: User): void {
        currentUser = { ...user };
        isEditMode = true;
        showUserModal = true;
    }
    
    // Function to save user (create or update)
    function saveUser(): void {
        if (isEditMode) {
            // Update existing user
            const index = users.findIndex(u => u.id === currentUser.id);
            if (index !== -1) {
                users[index] = { ...currentUser };
            }
        } else {
            // Add new user with the next available ID
            const newId = Math.max(...users.map(u => u.id), 0) + 1;
            users = [...users, { ...currentUser, id: newId }];
        }
        
        // Close the modal
        showUserModal = false;
    }
    
    // Function to delete a user
    function deleteUser(id: number): void {
        if (confirm("Are you sure you want to delete this user?")) {
            users = users.filter(user => user.id !== id);
            
            // If we're on a page that no longer has items, go to the previous page
            if (paginatedUsers.length === 1 && currentPage > 1) {
                currentPage--;
            }
        }
    }
    
    // Handle page change
    function handlePageChange(page: number): void {
        currentPage = page;
    }
    
    // Close modal
    function closeModal(): void {
        showUserModal = false;
    }
    
    // Define available roles for dropdown
    const roles: string[] = ["Admin", "User", "Support", "Guest"];
    
    // Handle authentication effect
    $: if (!isLoading && !isLoggedIn) {
        goto('/');
    }
</script>

{#if isLoading}
    <div class="flex items-center justify-center h-screen">
        <div class="animate-spin rounded-full h-12 w-12 border-t-2 border-b-2 border-blue-600"></div>
    </div>
{:else if isLoggedIn}
    <div class="p-6 max-w-6xl mx-auto">
        <div class="bg-white dark:bg-[#161b22] shadow-sm rounded-lg p-6">
            <div class="flex justify-between items-center mb-6">
                <h1 class="text-2xl font-bold">Users Management</h1>
                <button 
                    on:click={addUser}
                    class="bg-blue-600 hover:bg-blue-700 text-white px-4 py-2 rounded-md text-sm font-medium"
                >
                    Add User
                </button>
            </div>
            
            <!-- Search component -->
            <SearchBar bind:searchTerm={searchTerm} />
            
            <!-- Users Table component -->
            <UsersTable 
                users={paginatedUsers} 
                onEdit={editUser} 
                onDelete={deleteUser} 
            />
            
            <!-- Pagination component -->
            <Pagination 
                currentPage={currentPage}
                totalPages={totalPages}
                totalItems={filteredUsers.length}
                itemsPerPage={itemsPerPage}
                onPageChange={handlePageChange}
            />
        </div>
    </div>
    
    <!-- User Modal component -->
    <UserModal 
        showModal={showUserModal}
        isEditMode={isEditMode}
        user={currentUser}
        roles={roles}
        onSave={saveUser}
        onClose={closeModal}
    />
{/if}