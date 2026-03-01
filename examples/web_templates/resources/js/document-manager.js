/**
 * Document Manager - Manages document list and operations
 * Depends on: api-client.js
 * Used by: main.js
 */

const DocumentManager = {
    container: '#documents-list',
    
    async loadDocuments() {
        try {
            const documents = await ApiClient.get('/documents');
            this.renderDocuments(documents);
        } catch (error) {
            console.error('Failed to load documents:', error);
        }
    },
    
    renderDocuments(documents) {
        const html = documents.map(doc => `
            <div class="document-item" data-doc-id="${doc.id}">
                <h5>${doc.title}</h5>
                <p>Owner: ${doc.owner}</p>
                <p>Created: ${doc.createdDate}</p>
                <button onclick="DocumentManager.viewDocument(${doc.id})">View</button>
                <button onclick="DocumentManager.downloadDocument(${doc.id})">Download</button>
            </div>
        `).join('');
        
        $(this.container).html(html);
    },
    
    async viewDocument(docId) {
        try {
            const doc = await ApiClient.get(`/documents/${docId}`);
            console.log('Viewing document:', doc);
        } catch (error) {
            console.error('Failed to view document:', error);
        }
    },
    
    async downloadDocument(docId) {
        window.location.href = `/api/v1/documents/${docId}/download`;
    }
};
