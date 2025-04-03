<script lang="ts">
  import { User, Lock, LogIn } from 'lucide-svelte';
</script>

<div class="flex justify-center items-center min-h-[calc(100vh-120px)]">
  <div class="w-full max-w-md p-8 space-y-8 rounded-lg border border-github-light-border dark:border-github-dark-border bg-white dark:bg-github-dark-bg shadow-md">
    <div class="text-center">
      <h1 class="text-3xl font-bold text-github-light-text dark:text-github-dark-text">Welcome Back</h1>
      <p class="mt-2 text-github-light-secondary dark:text-github-dark-secondary">Please sign in to your account</p>
    </div>
    
    <form class="mt-8 space-y-6">
      <div class="space-y-4">
        <div class="space-y-2">
          <label for="email" class="block text-sm font-medium text-github-light-text dark:text-github-dark-text">Email or Username</label>
          <div class="relative">
            <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
              <User size={18} class="text-github-light-secondary dark:text-github-dark-secondary" />
            </div>
            <input 
              id="email" 
              name="email" 
              type="text" 
              required 
              class="block w-full pl-10 pr-3 py-2 border border-github-light-border dark:border-github-dark-border rounded-md bg-github-light-input dark:bg-github-dark-input text-github-light-text dark:text-github-dark-text focus:ring-2 focus:ring-github-light-accent dark:focus:ring-github-dark-accent focus:outline-none" 
              placeholder="Enter your email or username" 
            />
          </div>
        </div>
        
        <div class="space-y-2">
          <label for="password" class="block text-sm font-medium text-github-light-text dark:text-github-dark-text">Password</label>
          <div class="relative">
            <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
              <Lock size={18} class="text-github-light-secondary dark:text-github-dark-secondary" />
            </div>
            <input 
              id="password" 
              name="password" 
              type="password" 
              required 
              class="block w-full pl-10 pr-3 py-2 border border-github-light-border dark:border-github-dark-border rounded-md bg-github-light-input dark:bg-github-dark-input text-github-light-text dark:text-github-dark-text focus:ring-2 focus:ring-github-light-accent dark:focus:ring-github-dark-accent focus:outline-none" 
              placeholder="Enter your password" 
            />
          </div>
        </div>
      </div>

      <div class="flex items-center justify-between">
        <div class="flex items-center">
          <input 
            id="remember-me" 
            name="remember-me" 
            type="checkbox" 
            class="h-4 w-4 text-github-light-primary dark:text-github-dark-primary border-github-light-border dark:border-github-dark-border rounded focus:ring-github-light-accent dark:focus:ring-github-dark-accent" 
          />
          <label for="remember-me" class="ml-2 block text-sm text-github-light-text dark:text-github-dark-text">
            Remember me
          </label>
        </div>
        
        <div class="text-sm">
          <a href="#" class="font-medium text-github-light-primary dark:text-github-dark-primary hover:underline">
            Forgot password?
          </a>
        </div>
      </div>

      <div>
        <button 
          type="submit" 
          class="group relative w-full flex justify-center py-2 px-4 border border-transparent rounded-md font-medium text-white bg-github-light-primary dark:bg-github-dark-primary hover:opacity-90 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-github-light-accent dark:focus:ring-github-dark-accent transition-colors"
        >
          <span class="absolute left-0 inset-y-0 flex items-center pl-3">
            <LogIn size={18} class="text-white" />
          </span>
          Sign in
        </button>
      </div>
      
      <div class="text-center text-sm text-github-light-text dark:text-github-dark-text">
        Don't have an account?
        <a href="#" class="font-medium text-github-light-primary dark:text-github-dark-primary hover:underline ml-1">
          Create one
        </a>
      </div>
    </form>
  </div>
</div>
