package com.example.portal.fo.web.portlets.synthese;

import java.io.Serializable;

/**
 * Bean pour gérer la session dans le contexte "Gestion Épargne"
 */
public class GestionEPSessionBean implements Serializable {
    private static final long serialVersionUID = 1L;

    private String userId;
    private String sessionToken;

    public String getUserId() {
        return userId;
    }

    public void setUserId(String userId) {
        this.userId = userId;
    }
}
