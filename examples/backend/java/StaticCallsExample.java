/**
 * Example demonstrating static method calls and singleton pattern
 * This class shows how ServiceA and ServiceB use ServiceC static methods
 */

package backend.java;

public class StaticCallsExample {
    
    /**
     * Demonstrate direct static calls to ServiceC
     */
    public static void demonstrateStaticCalls() {
        System.out.println("\n=== STATIC CALLS DEMONSTRATION ===\n");
        
        // Direct static calls to ServiceC
        String data = "HelloWorld";
        
        // Log processing
        ServiceC.logProcessing("StaticCallsExample", "Starting static calls demonstration");
        
        // Validate format
        boolean isValid = ServiceC.isValidDataFormat(data);
        System.out.println("Data format validation: " + isValid);
        
        // Generate unique ID
        String uniqueId = ServiceC.generateUniqueId("DEMO");
        System.out.println("Generated unique ID: " + uniqueId);
        
        // Encode data
        String encoded = ServiceC.encodeData(data);
        System.out.println("Encoded data: " + encoded);
        
        // Compute checksum
        int checksum = ServiceC.computeChecksum(data);
        System.out.println("Checksum: " + checksum);
        
        // Batch transformation
        String[] items = {"First", "Second", "Third"};
        String[] transformed = ServiceC.transformBatch(items);
        System.out.println("Batch transformation completed");
        
        ServiceC.logProcessing("StaticCallsExample", "Static calls demonstration completed");
    }
    
    /**
     * Demonstrate ServiceA calling ServiceC static methods
     */
    public static void demonstrateServiceAWithServiceC() {
        System.out.println("\n=== SERVICE A WITH SERVICE C ===\n");
        
        ServiceA serviceA = new ServiceA();
        
        // ProcessData uses ServiceC static methods internally
        String result = serviceA.processData("ServiceATestData");
        System.out.println("ServiceA result: " + result);
        
        // ExecuteWorkflow uses ServiceC static methods for checksum
        serviceA.executeWorkflow("WorkflowTest");
        
        // BatchProcess uses ServiceC static methods
        String[] items = {"Item1", "Item2", "Item3"};
        serviceA.batchProcess(items);
    }
    
    /**
     * Demonstrate ServiceB calling ServiceC static methods
     */
    public static void demonstrateServiceBWithServiceC() {
        System.out.println("\n=== SERVICE B WITH SERVICE C ===\n");
        
        ServiceB serviceB = new ServiceB();
        
        // TransformData uses ServiceC static methods for encoding
        String result = serviceB.transformData("ServiceBTestData");
        System.out.println("ServiceB transformation result: " + result);
        
        // ValidateResult uses ServiceC static methods for format validation
        boolean isValid = serviceB.validateResult(result);
        System.out.println("Validation result: " + isValid);
        
        // ProcessComplexData uses ServiceC static methods
        String complexResult = serviceB.processComplexData("ComplexData", false);
        System.out.println("Complex data result: " + complexResult);
    }
    
    /**
     * Demonstrate singleton pattern with ServiceC
     */
    public static void demonstrateSingleton() {
        System.out.println("\n=== SINGLETON PATTERN DEMONSTRATION ===\n");
        
        // Get singleton instance
        ServiceCSingleton singleton1 = ServiceCSingleton.getInstance();
        ServiceCSingleton singleton2 = ServiceCSingleton.getInstance();
        
        // Verify it's the same instance
        System.out.println("Same instance? " + (singleton1 == singleton2));
        
        // Use singleton methods
        singleton1.encodeData("Test1");
        singleton1.generateUniqueId("SINGLETON");
        singleton1.encodeData("Test2");
        
        // Check call count
        System.out.println("Total tracked calls: " + singleton2.getCallCount());
    }
    
    /**
     * Main method demonstrating all static and singleton patterns
     */
    public static void main(String[] args) {
        System.out.println("========================================");
        System.out.println("  STATIC CALLS AND SINGLETON EXAMPLE");
        System.out.println("========================================");
        
        // Run all demonstrations
        demonstrateStaticCalls();
        demonstrateServiceAWithServiceC();
        demonstrateServiceBWithServiceC();
        demonstrateSingleton();
        
        System.out.println("\n========================================");
        System.out.println("  DEMONSTRATION COMPLETED");
        System.out.println("========================================\n");
    }
}
