/**
 * ServiceC Singleton - A singleton instance of ServiceC
 * Demonstrates static access patterns and singleton pattern
 */

package backend.java;

public class ServiceCSingleton {
    // Singleton instance
    private static ServiceCSingleton instance = null;
    private int callCount = 0;
    
    /**
     * Private constructor for singleton pattern
     */
    private ServiceCSingleton() {
        ServiceC.logProcessing("ServiceCSingleton", "Singleton instance created");
    }
    
    /**
     * Get the singleton instance
     */
    public static synchronized ServiceCSingleton getInstance() {
        if (instance == null) {
            instance = new ServiceCSingleton();
        }
        return instance;
    }
    
    /**
     * Track method calls
     */
    public void trackMethodCall(String methodName) {
        callCount++;
        String message = "Method call tracked: " + methodName + " (Total calls: " + callCount + ")";
        ServiceC.logProcessing("ServiceCSingleton", message);
    }
    
    /**
     * Wrap ServiceC.encodeData call
     */
    public String encodeData(String data) {
        trackMethodCall("encodeData");
        return ServiceC.encodeData(data);
    }
    
    /**
     * Wrap ServiceC.decodeData call
     */
    public String decodeData(String encodedData) {
        trackMethodCall("decodeData");
        return ServiceC.decodeData(encodedData);
    }
    
    /**
     * Wrap ServiceC.generateUniqueId call
     */
    public String generateUniqueId(String prefix) {
        trackMethodCall("generateUniqueId");
        return ServiceC.generateUniqueId(prefix);
    }
    
    /**
     * Get the total number of tracked calls
     */
    public int getCallCount() {
        return callCount;
    }
}
