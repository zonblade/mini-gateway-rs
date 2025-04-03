/**
 * Theme utility for handling OS dark/light mode preferences
 */

/**
 * Applies the theme (dark or light) to the document
 * @param isDark - Whether to apply dark mode
 */
export function applyTheme(isDark: boolean): void {
  document.documentElement.classList.toggle("dark", isDark);
}

/**
 * Initializes theme based on OS preference
 * @returns Current dark mode state
 */
export function initTheme(): boolean {
  // Check OS preference for color scheme
  const isDarkMode = window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches;
  
  // Apply theme based on OS preference
  applyTheme(isDarkMode);
  
  return isDarkMode;
}

/**
 * Sets up a listener for OS theme preference changes
 * @param callback - Callback function that receives the dark mode state
 * @returns Function to remove the listener
 */
export function setupThemeListener(callback: (isDark: boolean) => void): () => void {
  const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
  
  const listener = (event: MediaQueryListEvent): void => {
    callback(event.matches);
  };
  
  mediaQuery.addEventListener('change', listener);
  
  // Return function to remove listener
  return () => mediaQuery.removeEventListener('change', listener);
}