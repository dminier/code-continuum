package com.example.portal.fo.beans;

/**
 * Contexte de session singleton
 */
public class SessionContext {

    private static SessionContext instance;
    private String sessionId;

    private SessionContext() {
    }

    public static SessionContext getInstance() {
        if (instance == null) {
            instance = new SessionContext();
        }
        return instance;
    }

    public String getSessionId() {
        return sessionId;
    }
}
