// API client for backend communication

const API_BASE = process.env.REACT_APP_API_URL || 'http://localhost:8080';

export async function fetchUser(userId) {
    const response = await fetch(`${API_BASE}/users/${userId}`);
    if (!response.ok) {
        throw new Error(`Failed to fetch user: ${response.statusText}`);
    }
    return response.json();
}

export async function createUser(userData) {
    const response = await fetch(`${API_BASE}/users`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(userData)
    });
    return response.json();
}

export async function updateUser(userId, userData) {
    const response = await fetch(`${API_BASE}/users/${userId}`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(userData)
    });
    return response.json();
}

export async function deleteUser(userId) {
    const response = await fetch(`${API_BASE}/users/${userId}`, {
        method: 'DELETE'
    });
    return response.json();
}

export function createApiClient(baseUrl) {
    return {
        get: (path) => fetch(`${baseUrl}${path}`),
        post: (path, data) => fetch(`${baseUrl}${path}`, {
            method: 'POST',
            body: JSON.stringify(data)
        })
    };
}
