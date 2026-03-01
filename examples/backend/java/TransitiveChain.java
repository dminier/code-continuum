/**
 * Example demonstrating transitive class calls with depth of 10 in Java.
 * Each service calls the next one in the chain.
 */

package backend.java;

public class TransitiveChain {
    
    /**
     * Level 1 Service - Entry point of the chain
     */
    public static class Level1Service {
        private Level2Service nextLevel;
        
        public Level1Service() {
            this.nextLevel = new Level2Service();
        }
        
        public String execute(String input) {
            System.out.println("Level 1: Processing " + input);
            String result = nextLevel.execute(input + " -> L1");
            return result;
        }
        
        public int getDepth() {
            return 1 + nextLevel.getDepth();
        }
    }
    
    /**
     * Level 2 Service
     */
    public static class Level2Service {
        private Level3Service nextLevel;
        
        public Level2Service() {
            this.nextLevel = new Level3Service();
        }
        
        public String execute(String input) {
            System.out.println("Level 2: Processing " + input);
            return nextLevel.execute(input + " -> L2");
        }
        
        public int getDepth() {
            return 1 + nextLevel.getDepth();
        }
    }
    
    /**
     * Level 3 Service
     */
    public static class Level3Service {
        private Level4Service nextLevel;
        
        public Level3Service() {
            this.nextLevel = new Level4Service();
        }
        
        public String execute(String input) {
            System.out.println("Level 3: Processing " + input);
            return nextLevel.execute(input + " -> L3");
        }
        
        public int getDepth() {
            return 1 + nextLevel.getDepth();
        }
    }
    
    /**
     * Level 4 Service
     */
    public static class Level4Service {
        private Level5Service nextLevel;
        
        public Level4Service() {
            this.nextLevel = new Level5Service();
        }
        
        public String execute(String input) {
            System.out.println("Level 4: Processing " + input);
            return nextLevel.execute(input + " -> L4");
        }
        
        public int getDepth() {
            return 1 + nextLevel.getDepth();
        }
    }
    
    /**
     * Level 5 Service
     */
    public static class Level5Service {
        private Level6Service nextLevel;
        
        public Level5Service() {
            this.nextLevel = new Level6Service();
        }
        
        public String execute(String input) {
            System.out.println("Level 5: Processing " + input);
            return nextLevel.execute(input + " -> L5");
        }
        
        public int getDepth() {
            return 1 + nextLevel.getDepth();
        }
    }
    
    /**
     * Level 6 Service
     */
    public static class Level6Service {
        private Level7Service nextLevel;
        
        public Level6Service() {
            this.nextLevel = new Level7Service();
        }
        
        public String execute(String input) {
            System.out.println("Level 6: Processing " + input);
            return nextLevel.execute(input + " -> L6");
        }
        
        public int getDepth() {
            return 1 + nextLevel.getDepth();
        }
    }
    
    /**
     * Level 7 Service
     */
    public static class Level7Service {
        private Level8Service nextLevel;
        
        public Level7Service() {
            this.nextLevel = new Level8Service();
        }
        
        public String execute(String input) {
            System.out.println("Level 7: Processing " + input);
            return nextLevel.execute(input + " -> L7");
        }
        
        public int getDepth() {
            return 1 + nextLevel.getDepth();
        }
    }
    
    /**
     * Level 8 Service
     */
    public static class Level8Service {
        private Level9Service nextLevel;
        
        public Level8Service() {
            this.nextLevel = new Level9Service();
        }
        
        public String execute(String input) {
            System.out.println("Level 8: Processing " + input);
            return nextLevel.execute(input + " -> L8");
        }
        
        public int getDepth() {
            return 1 + nextLevel.getDepth();
        }
    }
    
    /**
     * Level 9 Service
     */
    public static class Level9Service {
        private Level10Service nextLevel;
        
        public Level9Service() {
            this.nextLevel = new Level10Service();
        }
        
        public String execute(String input) {
            System.out.println("Level 9: Processing " + input);
            return nextLevel.execute(input + " -> L9");
        }
        
        public int getDepth() {
            return 1 + nextLevel.getDepth();
        }
    }
    
    /**
     * Level 10 Service - Final level
     */
    public static class Level10Service {
        
        public String execute(String input) {
            System.out.println("Level 10: Final processing " + input);
            return input + " -> L10 [COMPLETE]";
        }
        
        public int getDepth() {
            return 1;
        }
    }
    
    /**
     * Chain Coordinator - Orchestrates the entire chain
     */
    public static class ChainCoordinator {
        private Level1Service entryPoint;
        
        public ChainCoordinator() {
            this.entryPoint = new Level1Service();
        }
        
        public String startChain(String initialData) {
            int depth = entryPoint.getDepth();
            System.out.println("Starting chain with depth: " + depth);
            String result = entryPoint.execute(initialData);
            System.out.println("Chain completed: " + result);
            return result;
        }
    }
    
    /**
     * Main method to demonstrate the transitive chain
     */
    public static void main(String[] args) {
        ChainCoordinator coordinator = new ChainCoordinator();
        coordinator.startChain("BEGIN");
    }
}
