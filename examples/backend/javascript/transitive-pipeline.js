/**
 * Example demonstrating transitive class calls with depth of 10 in JavaScript.
 * Each transformer calls the next one in the pipeline.
 */

/**
 * Level 1 Transformer - Entry point
 */
class Level1Transformer {
    constructor() {
        this.name = 'Level1';
        this.next = new Level2Transformer();
    }
    
    transform(data) {
        console.log(`Level 1: Transforming ${data}`);
        return this.next.transform(`${data} -> L1`);
    }
    
    getDepth() {
        return 1 + this.next.getDepth();
    }
}

/**
 * Level 2 Transformer
 */
class Level2Transformer {
    constructor() {
        this.name = 'Level2';
        this.next = new Level3Transformer();
    }
    
    transform(data) {
        console.log(`Level 2: Transforming ${data}`);
        return this.next.transform(`${data} -> L2`);
    }
    
    getDepth() {
        return 1 + this.next.getDepth();
    }
}

/**
 * Level 3 Transformer
 */
class Level3Transformer {
    constructor() {
        this.name = 'Level3';
        this.next = new Level4Transformer();
    }
    
    transform(data) {
        console.log(`Level 3: Transforming ${data}`);
        return this.next.transform(`${data} -> L3`);
    }
    
    getDepth() {
        return 1 + this.next.getDepth();
    }
}

/**
 * Level 4 Transformer
 */
class Level4Transformer {
    constructor() {
        this.name = 'Level4';
        this.next = new Level5Transformer();
    }
    
    transform(data) {
        console.log(`Level 4: Transforming ${data}`);
        return this.next.transform(`${data} -> L4`);
    }
    
    getDepth() {
        return 1 + this.next.getDepth();
    }
}

/**
 * Level 5 Transformer
 */
class Level5Transformer {
    constructor() {
        this.name = 'Level5';
        this.next = new Level6Transformer();
    }
    
    transform(data) {
        console.log(`Level 5: Transforming ${data}`);
        return this.next.transform(`${data} -> L5`);
    }
    
    getDepth() {
        return 1 + this.next.getDepth();
    }
}

/**
 * Level 6 Transformer
 */
class Level6Transformer {
    constructor() {
        this.name = 'Level6';
        this.next = new Level7Transformer();
    }
    
    transform(data) {
        console.log(`Level 6: Transforming ${data}`);
        return this.next.transform(`${data} -> L6`);
    }
    
    getDepth() {
        return 1 + this.next.getDepth();
    }
}

/**
 * Level 7 Transformer
 */
class Level7Transformer {
    constructor() {
        this.name = 'Level7';
        this.next = new Level8Transformer();
    }
    
    transform(data) {
        console.log(`Level 7: Transforming ${data}`);
        return this.next.transform(`${data} -> L7`);
    }
    
    getDepth() {
        return 1 + this.next.getDepth();
    }
}

/**
 * Level 8 Transformer
 */
class Level8Transformer {
    constructor() {
        this.name = 'Level8';
        this.next = new Level9Transformer();
    }
    
    transform(data) {
        console.log(`Level 8: Transforming ${data}`);
        return this.next.transform(`${data} -> L8`);
    }
    
    getDepth() {
        return 1 + this.next.getDepth();
    }
}

/**
 * Level 9 Transformer
 */
class Level9Transformer {
    constructor() {
        this.name = 'Level9';
        this.next = new Level10Transformer();
    }
    
    transform(data) {
        console.log(`Level 9: Transforming ${data}`);
        return this.next.transform(`${data} -> L9`);
    }
    
    getDepth() {
        return 1 + this.next.getDepth();
    }
}

/**
 * Level 10 Transformer - Final level
 */
class Level10Transformer {
    constructor() {
        this.name = 'Level10';
    }
    
    transform(data) {
        console.log(`Level 10: Final transformation ${data}`);
        return `${data} -> L10 [COMPLETE]`;
    }
    
    getDepth() {
        return 1;
    }
}

/**
 * Pipeline Orchestrator - Manages the transformation pipeline
 */
class PipelineOrchestrator {
    constructor() {
        this.transformer = new Level1Transformer();
    }
    
    runPipeline(initialData) {
        const depth = this.transformer.getDepth();
        console.log(`Starting transformation pipeline with depth: ${depth}`);
        const result = this.transformer.transform(initialData);
        console.log(`Pipeline completed: ${result}`);
        return result;
    }
}

// Main execution
if (typeof module !== 'undefined' && module.exports) {
    module.exports = {
        PipelineOrchestrator,
        Level1Transformer,
        Level2Transformer,
        Level3Transformer,
        Level4Transformer,
        Level5Transformer,
        Level6Transformer,
        Level7Transformer,
        Level8Transformer,
        Level9Transformer,
        Level10Transformer
    };
}

// Run example
if (require.main === module) {
    const orchestrator = new PipelineOrchestrator();
    orchestrator.runPipeline('BEGIN');
}
