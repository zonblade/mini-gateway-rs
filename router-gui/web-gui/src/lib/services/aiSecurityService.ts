import { user } from '$lib/stores/userStore';
import type { AiModel, ModelType, UpdateModelRequest, DeleteModelResponse, InferenceStats } from '$lib/types/aiSecurity';

// Helper function to get the current API base URL from the user store
function getApiBaseUrl(): string {
    let apiUrl: string = '';
    user.subscribe(value => {
        apiUrl = value?.api_base_url || '/api/v1';
    })();
    return apiUrl;
}

// Helper function to get the auth token from the store
function getAuthToken(): string | null {
    let token: string | null = null;
    user.subscribe(value => {
        token = value?.token || null;
    })();
    return token;
}

// Helper function to create request headers with auth token
function getHeaders(): Record<string, string> {
    const token = getAuthToken();
    const headers: Record<string, string> = {
        'Content-Type': 'application/json',
    };
    if (token) {
        headers['Authorization'] = `Bearer ${token}`;
    }
    return headers;
}

// Helper for auth header only (for multipart)
function getAuthHeader(): Record<string, string> {
    const token = getAuthToken();
    const headers: Record<string, string> = {};
    if (token) {
        headers['Authorization'] = `Bearer ${token}`;
    }
    return headers;
}

// AI Security API service
export const aiSecurityService = {
    // GET /ai-security/models - List all models
    listModels: async (): Promise<AiModel[]> => {
        try {
            const baseUrl = getApiBaseUrl();
            const response = await fetch(`${baseUrl}/ai-security/models`, {
                method: 'GET',
                headers: getHeaders()
            });
            if (!response.ok) {
                throw new Error(`Error fetching models: ${response.statusText}`);
            }
            return await response.json();
        } catch (error) {
            console.error('Failed to fetch AI models:', error);
            return [];
        }
    },

    // GET /ai-security/model/{id} - Get single model
    getModel: async (id: string): Promise<AiModel> => {
        const baseUrl = getApiBaseUrl();
        const response = await fetch(`${baseUrl}/ai-security/model/${id}`, {
            method: 'GET',
            headers: getHeaders()
        });
        if (!response.ok) {
            throw new Error(`Error fetching model: ${response.statusText}`);
        }
        return await response.json();
    },

    // POST /ai-security/model/upload - Upload new model (multipart form)
    uploadModel: async (name: string, modelType: ModelType, file: File, version?: string): Promise<AiModel> => {
        const baseUrl = getApiBaseUrl();
        const formData = new FormData();
        formData.append('name', name);
        formData.append('model_type', modelType);
        formData.append('file', file);
        if (version) {
            formData.append('version', version);
        }

        const response = await fetch(`${baseUrl}/ai-security/model/upload`, {
            method: 'POST',
            headers: getAuthHeader(), // No Content-Type for multipart
            body: formData
        });
        if (!response.ok) {
            const errorText = await response.text();
            throw new Error(`Error uploading model: ${errorText || response.statusText}`);
        }
        return await response.json();
    },

    // POST /ai-security/model/set - Update model config
    updateModel: async (request: UpdateModelRequest): Promise<AiModel> => {
        const baseUrl = getApiBaseUrl();
        const response = await fetch(`${baseUrl}/ai-security/model/set`, {
            method: 'POST',
            headers: getHeaders(),
            body: JSON.stringify(request)
        });
        if (!response.ok) {
            throw new Error(`Error updating model: ${response.statusText}`);
        }
        return await response.json();
    },

    // DELETE /ai-security/model/{id} - Delete model
    deleteModel: async (id: string): Promise<DeleteModelResponse> => {
        const baseUrl = getApiBaseUrl();
        const response = await fetch(`${baseUrl}/ai-security/model/${id}`, {
            method: 'DELETE',
            headers: getHeaders()
        });
        if (!response.ok) {
            throw new Error(`Error deleting model: ${response.statusText}`);
        }
        return await response.json();
    },

    // GET /ai-security/stats - Get inference statistics
    getInferenceStats: async (): Promise<InferenceStats> => {
        try {
            const baseUrl = getApiBaseUrl();
            const response = await fetch(`${baseUrl}/ai-security/stats`, {
                method: 'GET',
                headers: getHeaders()
            });
            if (!response.ok) {
                throw new Error(`Error fetching stats: ${response.statusText}`);
            }
            return await response.json();
        } catch (error) {
            console.error('Failed to fetch inference stats:', error);
            return { total_models: 0, enabled_models: 0, total_inferences: 0, models: [] };
        }
    }
};

export default aiSecurityService;
