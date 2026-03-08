/**
 * AI Security type definitions
 */

/** Model types supported by the AI security system */
export type ModelType = 'xgboost' | 'isolation';

/** AI Model entity from the backend */
export interface AiModel {
    /** Unique identifier (UUID) */
    id: string;
    /** Human-readable model name */
    name: string;
    /** Type of model */
    model_type: ModelType;
    /** Path to ONNX file on server */
    file_path: string;
    /** Whether model is enabled for inference */
    enabled: boolean;
    /** Version string (optional) */
    version: string | null;
    /** ISO-8601 upload timestamp */
    uploaded_at: string;
    /** ISO-8601 last inference timestamp (optional) */
    last_inference: string | null;
    /** Total number of inferences performed */
    inference_count: number;
}

/** Model statistics for inference stats endpoint */
export interface ModelStats {
    id: string;
    name: string;
    model_type: string;
    enabled: boolean;
    inference_count: number;
    last_inference: string | null;
}

/** Aggregated inference statistics */
export interface InferenceStats {
    total_models: number;
    enabled_models: number;
    total_inferences: number;
    models: ModelStats[];
}

/** Request body for updating a model */
export interface UpdateModelRequest {
    id: string;
    name?: string;
    enabled?: boolean;
    version?: string;
}

/** Response from delete endpoint */
export interface DeleteModelResponse {
    message: string;
    id: string;
}
