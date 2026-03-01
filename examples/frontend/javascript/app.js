// Frontend application entry point
// Handles client-side routing and state management

import React from 'react';
import { Router } from 'react-router-dom';
import { fetchUser } from './api';
import { UserProfile } from './components';

export function App() {
    const [user, setUser] = React.useState(null);
    const [loading, setLoading] = React.useState(true);
    
    React.useEffect(() => {
        loadUserData();
    }, []);
    
    async function loadUserData() {
        try {
            const userData = await fetchUser(1);
            setUser(userData);
        } finally {
            setLoading(false);
        }
    }
    
    if (loading) {
        return <div>Loading...</div>;
    }
    
    return (
        <Router>
            <div className="app">
                <UserProfile user={user} />
            </div>
        </Router>
    );
}

export function initializeApp(config) {
    console.log('App initialized with config:', config);
}

export const version = "1.0.0";
