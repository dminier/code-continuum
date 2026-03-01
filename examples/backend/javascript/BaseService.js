class BaseService {
    constructor() {
        this.serviceName = "BaseService";
    }

    initialize() {
        console.log("Initializing BaseService");
    }

    getServiceName() {
        return this.serviceName;
    }
}

module.exports = BaseService;
