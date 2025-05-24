<script lang="ts">
    import { onMount } from "svelte";
    import { goto } from "$app/navigation";
    import { user } from "$lib/stores/userStore";
    import { usersStore } from "$lib/stores/usersStore";
    import { userActions } from "$lib/actions/userActions";
    import type { User } from "$lib/types/userTypes";
    import SearchBar from "$lib/components/users/SearchBar.svelte";
    import UsersTable from "$lib/components/users/UsersTable.svelte";
    import Pagination from "$lib/components/users/Pagination.svelte";
    import UserModal from "$lib/components/users/UserModal.svelte";
    import LoadingSpinner from "$lib/components/common/LoadingSpinner.svelte";

    // Authentication state
    let isLoggedIn = false;
    let isLoading = true;

    // Subscribe to both stores
    const unsubAuthCheck = user.subscribe((value) => {
        isLoggedIn = !!value;
        isLoading = false; // Set loading to false once we've checked auth status
    });

    // Destructure values from the users store
    $: ({
        paginatedUsers,
        isLoading: isLoadingUsers,
        isProcessing,
        error: errorMessage,
        searchTerm,
        currentPage,
        totalPages,
        filteredUsers,
        itemsPerPage,
    } = $usersStore);

    // Local error handling
    let localErrorMessage: string | null = null;

    onMount(() => {
        // Redirect happens after auth check is complete
        if (!isLoading && !isLoggedIn) {
            goto("/");
        } else if (isLoggedIn) {
            usersStore.loadUsers(); // Load users if logged in
        }

        return () => {
            unsubAuthCheck(); // Clean up subscription
        };
    });

    // For add/edit user popup
    let showUserModal = false;
    let isEditMode = false;
    let currentUser: User = {
        id: "",
        username: "",
        email: "",
        role: "User",
        active: true,
    };

    // Reference to the UserModal component to access password
    let userModalComponent: UserModal;

    // Function to open modal for adding a new user
    function addUser(): void {
        currentUser = {
            id: "",
            username: "",
            email: "",
            role: "User",
            active: true,
        };
        isEditMode = false;
        showUserModal = true;
        usersStore.clearError();
        localErrorMessage = null;
    }

    // Function to open modal for editing an existing user
    function editUser(user: User): void {
        currentUser = { ...user };
        isEditMode = true;
        showUserModal = true;
        usersStore.clearError();
        localErrorMessage = null;
    }

    // Function to save user (create or update) using userActions
    async function saveUser(): Promise<void> {
        try {
            if (isEditMode) {
                // Update existing user
                const userData = {
                    username: currentUser.username,
                    email: currentUser.email,
                    role: currentUser.role,
                    active: currentUser.active,
                };

                await userActions.updateUser(currentUser.id, userData);
            } else {
                // Add new user with password from modal
                const password = userModalComponent
                    ? userModalComponent.getPassword()
                    : "";

                if (!password) {
                    // Set local error message
                    localErrorMessage = "Password is required for new users";
                    return;
                }

                const userData = {
                    username: currentUser.username,
                    email: currentUser.email,
                    password: password,
                    role: currentUser.role,
                    active: currentUser.active,
                };

                await userActions.createUser(userData);
            }

            // Reload users after successful operation
            await usersStore.loadUsers();

            // Close the modal
            showUserModal = false;
        } catch (error) {
            // Display error message
            if (error instanceof Error) {
                localErrorMessage = error.message;
            } else {
                localErrorMessage = "An unknown error occurred";
            }
        }
    }

    // Function to delete a user using userActions
    async function deleteUser(id: string): Promise<void> {
        if (!confirm("Are you sure you want to delete this user?")) {
            return;
        }

        try {
            await userActions.deleteUser(id);
            // Reload users after deletion
            await usersStore.loadUsers();
        } catch (error) {
            // Update the store's error state through a regular store operation
            if (error instanceof Error) {
                localErrorMessage = error.message;
            } else {
                localErrorMessage = "Failed to delete user";
            }
        }
    }

    // Handle page change
    function handlePageChange(page: number): void {
        usersStore.setPage(page);
    }

    // Handle search term change
    function handleSearchChange(event: CustomEvent<string>): void {
        usersStore.setSearchTerm(event.detail);
    }

    // Close modal
    function closeModal(): void {
        showUserModal = false;
        localErrorMessage = null;
    }

    // Define available roles for dropdown
    const roles: string[] = ["admin", "staff", "user"];

    // Handle authentication effect
    $: if (!isLoading && !isLoggedIn) {
        goto("/");
    }
</script>

{#if isLoading}
    <LoadingSpinner />
{:else if isLoggedIn}
    <div class="p-6 mx-auto w-full flex flex-col items-center">
        <div class="w-full max-w-[900px]">
            <div class="flex justify-between items-center mb-6">
                <h1 class="text-2xl font-bold">Users Management</h1>
                <button
                    on:click={addUser}
                    class="bg-blue-600 hover:bg-blue-700 text-white px-4 py-2 text-sm font-medium border border-transparent"
                    disabled={isProcessing}
                >
                    Add User
                </button>
            </div>
            <div class="">
                {#if errorMessage || localErrorMessage}
                    <div class="mb-4 p-3 bg-red-100 text-red-700">
                        <p>{errorMessage || localErrorMessage}</p>
                    </div>
                {/if}

                <!-- Search component -->
                <SearchBar {searchTerm} on:search={handleSearchChange} />

                {#if isLoadingUsers}
                    <LoadingSpinner />
                {:else if paginatedUsers.length === 0}
                    <div class="text-center py-8 text-gray-500">
                        <p>No users found.</p>
                    </div>
                {:else}
                    <!-- Users Table component -->
                    <UsersTable
                        users={paginatedUsers}
                        onEdit={editUser}
                        onDelete={deleteUser}
                        disabled={isProcessing}
                    />

                    <!-- Pagination component -->
                    <Pagination
                        {currentPage}
                        {totalPages}
                        totalItems={filteredUsers.length}
                        {itemsPerPage}
                        onPageChange={handlePageChange}
                    />
                {/if}
            </div>
        </div>
    </div>

    <!-- User Modal component with bind:this to access its methods -->
    <UserModal
        bind:this={userModalComponent}
        showModal={showUserModal}
        {isEditMode}
        user={currentUser}
        {roles}
        onSave={saveUser}
        onClose={closeModal}
        {isProcessing}
        errorMessage={localErrorMessage || errorMessage}
    />
{/if}
