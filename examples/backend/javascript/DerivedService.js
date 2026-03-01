const BaseService = require('./BaseService');

class DerivedService extends BaseService {
    constructor() {
        super();
        this.serviceName = "DerivedService";
    }

    initialize() {
        super.initialize();
        console.log("Initializing DerivedService");
    }

    performAction() {
        console.log("Performing action in DerivedService");
    }
}

module.exports = DerivedService;
