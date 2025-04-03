import { writable } from 'svelte/store';
import { browser } from '$app/environment';

// Function to check if the system prefers dark mode
function getSystemPreference() {
  if (browser) {
    return window.matchMedia('(prefers-color-scheme: dark)').matches;
  }
  return false;
}

// Initialize the theme from localStorage or system preference
function createThemeStore() {
  // Default to system preference if no stored value
  const storedTheme = browser ? localStorage.getItem('theme') : null;
  const initialValue = storedTheme 
    ? storedTheme === 'dark' 
    : getSystemPreference();
  
  const { subscribe, set, update } = writable(initialValue);

  return {
    subscribe,
    toggleTheme: () => {
      update(isDark => {
        const newValue = !isDark;
        if (browser) {
          localStorage.setItem('theme', newValue ? 'dark' : 'light');
          // Apply or remove the 'dark' class from the document
          if (newValue) {
            document.documentElement.classList.add('dark');
          } else {
            document.documentElement.classList.remove('dark');
          }
        }
        return newValue;
      });
    },
    // Initialize the theme in the DOM
    initialize: () => {
      if (browser) {
        const isDark = storedTheme 
          ? storedTheme === 'dark' 
          : getSystemPreference();
        
        if (isDark) {
          document.documentElement.classList.add('dark');
        } else {
          document.documentElement.classList.remove('dark');
        }
      }
    }
  };
}

export const isDarkMode = createThemeStore();