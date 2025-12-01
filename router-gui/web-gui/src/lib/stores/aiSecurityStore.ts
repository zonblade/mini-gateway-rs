import { writable } from 'svelte/store';
import type { AiModel, InferenceStats, ModelType, UpdateModelRequest } from '$lib/types/aiSecurity';
import { aiSecurityService } from '$lib/services/aiSecurityService';

// Store state interface
interface AiSecurityState {
    models: AiModel[];
    stats: InferenceStats | null;
    isLoading: boolean;
    isUploading: boolean;
    isProcessing: boolean;
    error: string | null;
    selectedModel: AiModel | null;
}

// Initial state
const initialState: AiSecurityState = {
    models: [],
    stats: null,
    isLoading: false,
    isUploading: false,
    isProcessing: false,
    error: null,
    selectedModel: null
};

// Create the writable store
function createAiSecurityStore() {
    const { subscribe, set, update } = writable<AiSecurityState>(initialState);

    return {
        subscribe,

        // Reset the store to initial state
        reset: () => set(initialState),

        // Load all models from the API
        loadModels: async () => {
            update(state => ({ ...state, isLoading: true, error: null }));
            try {
                const models = await aiSecurityService.listModels();
                update(state => ({
                    ...state,
                    models,
                    isLoading: false
                }));
            } catch (error) {
                console.error('Failed to load AI models:', error);
                update(state => ({
                    ...state,
                    isLoading: false,
                    error: error instanceof Error ? error.message : 'Failed to load models'
                }));
            }
        },

        // Load inference statistics
        loadStats: async () => {
            try {
                const stats = await aiSecurityService.getInferenceStats();
                update(state => ({ ...state, stats }));
            } catch (error) {
                console.error('Failed to load inference stats:', error);
            }
        },

        // Upload a new model
        uploadModel: async (name: string, modelType: ModelType, file: File, version?: string) => {
            update(state => ({ ...state, isUploading: true, error: null }));
            try {
                const newModel = await aiSecurityService.uploadModel(name, modelType, file, version);
                update(state => ({
                    ...state,
                    models: [...state.models, newModel],
                    isUploading: false
                }));
                return true;
            } catch (error) {
                console.error('Failed to upload model:', error);
                update(state => ({
                    ...state,
                    isUploading: false,
                    error: error instanceof Error ? error.message : 'Failed to upload model'
                }));
                return false;
            }
        },

        // Update model (enable/disable, name, version)
        updateModel: async (request: UpdateModelRequest) => {
            update(state => ({ ...state, isProcessing: true, error: null }));
            try {
                const updatedModel = await aiSecurityService.updateModel(request);
                update(state => ({
                    ...state,
                    models: state.models.map(m => m.id === request.id ? updatedModel : m),
                    isProcessing: false
                }));
                return true;
            } catch (error) {
                console.error('Failed to update model:', error);
                update(state => ({
                    ...state,
                    isProcessing: false,
                    error: error instanceof Error ? error.message : 'Failed to update model'
                }));
                return false;
            }
        },

        // Toggle model enabled/disabled
        toggleModel: async (id: string, enabled: boolean) => {
            update(state => ({ ...state, isProcessing: true, error: null }));
            try {
                const updatedModel = await aiSecurityService.updateModel({ id, enabled });
                update(state => ({
                    ...state,
                    models: state.models.map(m => m.id === id ? updatedModel : m),
                    isProcessing: false
                }));
                return true;
            } catch (error) {
                console.error('Failed to toggle model:', error);
                update(state => ({
                    ...state,
                    isProcessing: false,
                    error: error instanceof Error ? error.message : 'Failed to toggle model'
                }));
                return false;
            }
        },

        // Delete a model
        deleteModel: async (id: string) => {
            update(state => ({ ...state, isProcessing: true, error: null }));
            try {
                await aiSecurityService.deleteModel(id);
                update(state => ({
                    ...state,
                    models: state.models.filter(m => m.id !== id),
                    isProcessing: false
                }));
                return true;
            } catch (error) {
                console.error('Failed to delete model:', error);
                update(state => ({
                    ...state,
                    isProcessing: false,
                    error: error instanceof Error ? error.message : 'Failed to delete model'
                }));
                return false;
            }
        },

        // Select a model for editing
        selectModel: (model: AiModel | null) => {
            update(state => ({ ...state, selectedModel: model }));
        },

        // Clear error message
        clearError: () => {
            update(state => ({ ...state, error: null }));
        }
    };
}

// Create and export the store
export const aiSecurityStore = createAiSecurityStore();
