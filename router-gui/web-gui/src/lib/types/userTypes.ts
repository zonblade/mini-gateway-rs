// User interfaces for the frontend
export interface User {
    id: string;
    username: string;
    email: string;
    role: string;
    active: boolean;
    created_at?: string;
    updated_at?: string;
}

// API response and request types
export interface ApiUser {
    id: string;
    username: string;
    email: string;
    role: string;
    created_at: string;
    updated_at: string;
}

export interface CreateUserRequest {
    username: string;
    email: string;
    password: string;
    role?: string;
}

export interface UpdateUserRequest {
    username?: string;
    email?: string;
    password?: string;
    role?: string;
}

export interface DeleteUserResponse {
    message: string;
}