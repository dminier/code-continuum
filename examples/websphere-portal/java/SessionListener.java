package com.example.listeners;

import javax.servlet.http.HttpSessionEvent;
import javax.servlet.http.HttpSessionListener;

/**
 * Session listener for session management
 */
public class SessionListener implements HttpSessionListener {
    
    private static int activeSessions = 0;
    
    @Override
    public void sessionCreated(HttpSessionEvent event) {
        activeSessions++;
        System.out.println("Session created: " + event.getSession().getId() + 
                         " (Active sessions: " + activeSessions + ")");
    }
    
    @Override
    public void sessionDestroyed(HttpSessionEvent event) {
        activeSessions--;
        System.out.println("Session destroyed: " + event.getSession().getId() + 
                         " (Active sessions: " + activeSessions + ")");
    }
    
    public static int getActiveSessions() {
        return activeSessions;
    }
}
