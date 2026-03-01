package com.example.portlets;

import javax.portlet.*;
import java.io.IOException;

/**
 * Base class for all portal portlets providing common functionality
 */
public abstract class BasePortlet extends GenericPortlet {
    
    protected PortletConfig portletConfig;
    protected PortletContext portletContext;
    
    @Override
    public void init(PortletConfig config) throws PortletException {
        super.init(config);
        this.portletConfig = config;
        this.portletContext = config.getPortletContext();
    }
    
    /**
     * Get initialization parameter from portlet.xml
     */
    protected String getInitParameter(String name) {
        return portletConfig.getInitParameter(name);
    }
    
    /**
     * Dispatch to JSP template
     */
    protected void dispatch(String template, RenderRequest request, RenderResponse response) 
            throws PortletException, IOException {
        PortletRequestDispatcher dispatcher = portletContext.getRequestDispatcher(template);
        if (dispatcher != null) {
            dispatcher.include(request, response);
        }
    }
    
    /**
     * Set render parameter for view transitions
     */
    protected void setRenderParameter(ActionRequest request, ActionResponse response, 
                                     String name, String value) {
        response.setRenderParameter(name, value);
    }
}
