/**
 * Service C - Utility service with static methods
 * Provides static helper methods for data processing and logging
 */

package backend.java;

public class ServiceC {
    /**
     * Static utility method for data logging
     */
    public static void logProcessing(String serviceName, String message) {
        System.out.println("[" + serviceName + "] StaticLog: " + message);
    }
    
    /**
     * Static utility method for data encryption/encoding
     */
    public static String encodeData(String data) {
        if (data == null) {
            return "ENCODED_NULL";
        }
        // Simple encoding: reverse and add prefix
        String reversed = new StringBuilder(data).reverse().toString();
        return "ENCODED_" + reversed;
    }
    
    /**
     * Static utility method for data decryption/decoding
     */
    public static String decodeData(String encodedData) {
        if (encodedData == null || !encodedData.startsWith("ENCODED_")) {
            return "DECODED_INVALID";
        }
        // Remove prefix
        String withoutPrefix = encodedData.substring(8);
        // Reverse back
        return new StringBuilder(withoutPrefix).reverse().toString();
    }
    
    /**
     * Static method to validate data format
     */
    public static boolean isValidDataFormat(String data) {
        return data != null && !data.isEmpty() && data.length() >= 3;
    }
    
    /**
     * Static method to generate unique ID
     */
    public static String generateUniqueId(String prefix) {
        long timestamp = System.currentTimeMillis();
        return prefix + "_" + timestamp;
    }
    
    /**
     * Static method for batch transformation
     */
    public static String[] transformBatch(String[] items) {
        if (items == null) {
            return new String[0];
        }
        
        String[] results = new String[items.length];
        for (int i = 0; i < items.length; i++) {
            results[i] = encodeData(items[i]);
        }
        ServiceC.logProcessing("ServiceC", "Batch transformation completed for " + items.length + " items");
        return results;
    }
    
    /**
     * Static method to compute checksum
     */
    public static int computeChecksum(String data) {
        if (data == null) {
            return 0;
        }
        int checksum = 0;
        for (char c : data.toCharArray()) {
            checksum += (int) c;
        }
        ServiceC.logProcessing("ServiceC", "Checksum computed: " + checksum);
        return checksum;
    }
}
