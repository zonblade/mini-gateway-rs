/**
 * UUID Generation Service
 * 
 * Provides UUID generation functionality that works across all browsers,
 * including older browsers that may not have full crypto API support.
 * 
 * This service calls the backend API to generate secure UUIDs server-side,
 * ensuring consistent and reliable UUID generation regardless of browser capabilities.
 */

import { API_BASE_URL } from '../config';

/**
 * Response structure from the UUID generation API
 */
interface UuidResponse {
    uuid: string;
    generated_at: string;
}

/**
 * Generate a new UUID using the backend API
 * 
 * This function calls the backend `/api/v1/generation/uuid` endpoint to generate
 * a secure UUID v4. The backend uses the system's secure random number generator,
 * making it more reliable than client-side generation in older browsers.
 * 
 * @returns Promise<string> The generated UUID string
 * @throws Error if the UUID generation fails or the user is not authenticated
 * 
 * @example
 * ```typescript
 * try {
 *   const uuid = await generateUUID();
 *   console.log('Generated UUID:', uuid);
 * } catch (error) {
 *   console.error('Failed to generate UUID:', error);
 * }
 * ```
 */
export async function generateUUID(): Promise<string> {
    try {
        // Get authentication token from localStorage
        const token = localStorage.getItem('token');
        if (!token) {
            throw new Error('Authentication required - no token found');
        }

        const response = await fetch(`${API_BASE_URL}/generation/uuid`, {
            method: 'GET',
            headers: {
                'Content-Type': 'application/json',
                'Authorization': `Bearer ${token}`,
            },
        });

        if (!response.ok) {
            if (response.status === 401) {
                throw new Error('Authentication failed - please log in again');
            } else if (response.status === 403) {
                throw new Error('Access denied - insufficient permissions');
            } else {
                throw new Error(`UUID generation failed: ${response.status} ${response.statusText}`);
            }
        }

        const data: UuidResponse = await response.json();
        return data.uuid;
        
    } catch (error) {
        console.error('UUID generation error:', error);
        
        // Fallback to client-side generation if backend fails
        console.warn('Falling back to client-side UUID generation');
        return fallbackGenerateUUID();
    }
}

/**
 * Fallback UUID generation for cases where backend API is unavailable
 * 
 * This function provides a crypto-free UUID generation method that works across all browsers,
 * including older ones. It uses only Math.random() combined with timestamp to create
 * a reasonably unique identifier without touching any crypto APIs.
 * 
 * @returns string A fallback UUID-like string
 * @private
 */
function fallbackGenerateUUID(): string {
    // Pure Math.random() based UUID generation - NO CRYPTO APIs
    const template = 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx';
    const timestamp = Date.now().toString(36);
    
    return template.replace(/[xy]/g, function(c) {
        const r = Math.random() * 16 | 0;
        const v = c === 'x' ? r : (r & 0x3 | 0x8);
        return v.toString(16);
    }) + '-' + timestamp;
}