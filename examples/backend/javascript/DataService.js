/**
 * DataService - Main service that orchestrates data processing
 * Calls ApiClient and Validator for operations
 */

class ApiClient {
    constructor() {
        this.baseUrl = 'https://api.example.com';
    }
    
    fetchData(endpoint) {
        console.log(`ApiClient: Fetching from ${endpoint}`);
        return `DATA_FROM_${endpoint}`;
    }
    
    postData(endpoint, data) {
        console.log(`ApiClient: Posting to ${endpoint}:`, data);
        return { success: true, id: 123 };
    }
    
    getStatus() {
        return 'ApiClient is ready';
    }
}

class Validator {
    constructor() {
        this.rules = [];
    }
    
    validateInput(data) {
        console.log('Validator: Checking input', data);
        return data !== null && data !== undefined && data.length > 0;
    }
    
    validateOutput(result) {
        console.log('Validator: Checking output', result);
        return result.includes('PROCESSED');
    }
    
    sanitize(input) {
        console.log('Validator: Sanitizing', input);
        return input.trim().replace(/[<>]/g, '');
    }
}

class DataService {
    constructor() {
        this.name = 'DataService';
        this.apiClient = new ApiClient();
        this.validator = new Validator();
    }
    
    /**
     * Process data - validates, fetches, and transforms
     */
    processData(input) {
        console.log('DataService: Processing', input);
        
        // Validate input
        if (!this.validator.validateInput(input)) {
            return 'ERROR: Invalid input';
        }
        
        // Sanitize
        const sanitized = this.validator.sanitize(input);
        
        // Fetch from API
        const rawData = this.apiClient.fetchData(sanitized);
        
        // Transform
        const processed = `PROCESSED_${rawData}`;
        
        console.log('DataService: Result =', processed);
        return processed;
    }
    
    /**
     * Execute complex workflow
     */
    executeWorkflow(data) {
        console.log('DataService: Starting workflow');
        
        // Step 1: Process
        const processed = this.processData(data);
        
        // Step 2: Validate output
        const isValid = this.validator.validateOutput(processed);
        
        if (isValid) {
            // Step 3: Save result
            const saved = this.apiClient.postData('results', processed);
            console.log('DataService: Workflow completed', saved);
            return saved;
        }
        
        console.log('DataService: Workflow validation failed');
        return null;
    }
    
    /**
     * Get status from all components
     */
    getStatus() {
        const myStatus = 'DataService is running';
        const apiStatus = this.apiClient.getStatus();
        return `${myStatus} | ${apiStatus}`;
    }
    
    /**
     * Batch process multiple items
     */
    batchProcess(items) {
        console.log(`DataService: Batch processing ${items.length} items`);
        
        const results = [];
        for (const item of items) {
            const result = this.processData(item);
            results.push(result);
        }
        
        // Notify completion via API
        this.apiClient.postData('batch-complete', { count: results.length });
        
        return results;
    }
}

/**
 * Main execution function
 */
function main() {
    console.log('=== DataService Test ===');
    
    const service = new DataService();
    
    // Test 1: Simple processing
    service.processData('TestData');
    
    // Test 2: Workflow
    service.executeWorkflow('WorkflowData');
    
    // Test 3: Status
    console.log(service.getStatus());
    
    // Test 4: Batch processing
    const items = ['Item1', 'Item2', 'Item3'];
    service.batchProcess(items);
}

// Export for testing
if (typeof module !== 'undefined' && module.exports) {
    module.exports = { DataService, ApiClient, Validator, main };
}

// Run main if executed directly
if (typeof require !== 'undefined' && require.main === module) {
    main();
}
