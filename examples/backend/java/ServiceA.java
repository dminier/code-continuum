/**
 * Service A - Calls Service B for data processing
 * Also uses static methods from Service C
 */

package backend.java;

public class ServiceA {
    private String name;
    private ServiceB serviceB;
    
    /**
     * Constructor initializing ServiceA with ServiceB dependency
     */
    public ServiceA() {
        this.name = "ServiceA";
        this.serviceB = new ServiceB();
    }
    
    /**
     * Process data by calling ServiceB and static methods from ServiceC
     */
    public String processData(String input) {
        System.out.println("ServiceA: Processing " + input);
        
        // Validate input first
        if (input == null || input.isEmpty()) {
            return "Error: Invalid input";
        }
        
        // Use ServiceC static method for format validation
        if (!ServiceC.isValidDataFormat(input)) {
            ServiceC.logProcessing("ServiceA", "Invalid input format detected");
            return "Error: Invalid input format";
        }
        
        // Generate unique ID using ServiceC static method
        String uniqueId = ServiceC.generateUniqueId("SA");
        ServiceC.logProcessing("ServiceA", "Processing with ID: " + uniqueId);
        
        // Call ServiceB for transformation
        String transformed = serviceB.transformData(input);
        
        // Additional processing
        String result = "ServiceA[" + transformed + "]";
        System.out.println("ServiceA: Result = " + result);
        
        return result;
    }
    
    /**
     * Execute a complex workflow involving ServiceB and ServiceC
     */
    public void executeWorkflow(String data) {
        System.out.println("ServiceA: Starting workflow");
        
        // Step 1: Process data
        String processed = processData(data);
        
        // Step 2: Validate with ServiceB
        boolean isValid = serviceB.validateResult(processed);
        
        // Use ServiceC static method to compute checksum
        int checksum = ServiceC.computeChecksum(processed);
        ServiceC.logProcessing("ServiceA", "Checksum for processed data: " + checksum);
        
        if (isValid) {
            System.out.println("ServiceA: Workflow completed successfully");
        } else {
            System.out.println("ServiceA: Workflow validation failed");
        }
    }
    
    /**
     * Get status from both services
     */
    public String getStatus() {
        String myStatus = "ServiceA is running";
        String otherStatus = serviceB.getStatus();
        return myStatus + " | " + otherStatus;
    }
    
    /**
     * Batch process multiple items using ServiceB and ServiceC
     */
    public void batchProcess(String[] items) {
        System.out.println("ServiceA: Batch processing " + items.length + " items");
        
        for (String item : items) {
            String result = serviceB.transformData(item);
            System.out.println("ServiceA: Processed " + item + " -> " + result);
        }
        
        // Use ServiceC static method for batch transformation and encoding
        String[] encodedItems = ServiceC.transformBatch(items);
        ServiceC.logProcessing("ServiceA", "Batch encoding completed with " + encodedItems.length + " results");
        
        serviceB.notifyCompletion("Batch processing completed");
    }
    
    /**
     * Main method for testing
     */
    public static void main(String[] args) {
        ServiceA serviceA = new ServiceA();
        
        // Test simple processing
        serviceA.processData("TestData");
        
        // Test workflow
        serviceA.executeWorkflow("WorkflowData");
        
        // Test status
        System.out.println(serviceA.getStatus());
        
        // Test batch processing
        String[] items = {"Item1", "Item2", "Item3"};
        serviceA.batchProcess(items);
    }
}
