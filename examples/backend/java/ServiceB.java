/**
 * Service B - Provides data transformation and validation
 * Also uses static methods from Service C
 * Can also call back to Service A for coordination
 */

package backend.java;

public class ServiceB {
    private String name;
    private int operationCount;
    
    /**
     * Constructor initializing ServiceB
     */
    public ServiceB() {
        this.name = "ServiceB";
        this.operationCount = 0;
    }
    
    /**
     * Transform input data and use ServiceC static methods
     */
    public String transformData(String input) {
        operationCount++;
        System.out.println("ServiceB: Transforming " + input);
        
        if (input == null) {
            return "NULL";
        }
        
        // Apply transformation
        String transformed = input.toUpperCase() + "_TRANSFORMED";
        System.out.println("ServiceB: Transformation result = " + transformed);
        
        // Use ServiceC static method for encoding
        String encoded = ServiceC.encodeData(transformed);
        ServiceC.logProcessing("ServiceB", "Encoded transformation: " + encoded);
        
        return encoded;
    }
    
    /**
     * Validate a result string using ServiceC static methods
     */
    public boolean validateResult(String result) {
        System.out.println("ServiceB: Validating " + result);
        
        if (result == null || result.isEmpty()) {
            return false;
        }
        
        // Check if result contains expected pattern
        boolean isValid = result.contains("ServiceA") && result.contains("TRANSFORMED");
        System.out.println("ServiceB: Validation result = " + isValid);
        
        // Use ServiceC static method to validate format
        boolean formatValid = ServiceC.isValidDataFormat(result);
        ServiceC.logProcessing("ServiceB", "Format validation: " + formatValid);
        
        return isValid && formatValid;
    }
    
    /**
     * Get current status
     */
    public String getStatus() {
        return "ServiceB is active (operations: " + operationCount + ")";
    }
    
    /**
     * Notify completion and potentially call ServiceA for coordination
     */
    public void notifyCompletion(String message) {
        System.out.println("ServiceB: Notification - " + message);
        
        // In a real scenario, this might call back to ServiceA
        // to coordinate complex workflows
        if (operationCount > 5) {
            coordinateWithServiceA(message);
        }
    }
    
    /**
     * Coordinate with ServiceA for complex operations
     */
    private void coordinateWithServiceA(String context) {
        System.out.println("ServiceB: Coordinating with ServiceA for: " + context);
        
        // Create ServiceA instance for coordination
        ServiceA serviceA = new ServiceA();
        
        // Call back to ServiceA
        String coordinationData = "COORDINATION_" + context;
        String result = serviceA.processData(coordinationData);
        
        System.out.println("ServiceB: Coordination completed with result: " + result);
    }
    
    /**
     * Process complex data by delegating to ServiceA when needed
     */
    public String processComplexData(String input, boolean useServiceA) {
        System.out.println("ServiceB: Processing complex data");
        
        String intermediate = transformData(input);
        
        // Use ServiceC static method to generate unique ID for complex operation
        String operationId = ServiceC.generateUniqueId("SB_COMPLEX");
        ServiceC.logProcessing("ServiceB", "Complex operation ID: " + operationId);
        
        if (useServiceA) {
            // Delegate to ServiceA for additional processing
            ServiceA serviceA = new ServiceA();
            return serviceA.processData(intermediate);
        }
        
        return intermediate;
    }
    
    /**
     * Reset operation counter
     */
    public void reset() {
        operationCount = 0;
        System.out.println("ServiceB: Operation counter reset");
    }
    
    /**
     * Main method for testing
     */
    public static void main(String[] args) {
        ServiceB serviceB = new ServiceB();
        
        // Test transformation
        String result = serviceB.transformData("TestData");
        
        // Test validation
        serviceB.validateResult(result);
        
        // Test status
        System.out.println(serviceB.getStatus());
        
        // Test complex processing with ServiceA
        serviceB.processComplexData("ComplexData", true);
        
        // Test notification
        serviceB.notifyCompletion("All tests completed");
    }
}
