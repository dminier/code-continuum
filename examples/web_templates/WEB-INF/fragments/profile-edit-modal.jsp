<%-- Profile edit modal fragment --%>
<%@ page contentType="text/html;charset=UTF-8" language="java" %>

<div id="profile-modal" class="modal" style="display: none;">
    <div class="modal-content">
        <span id="close-modal" class="close">&times;</span>
        <h2>Edit Profile</h2>
        
        <form id="profile-form">
            <div class="form-group">
                <label for="profile-name">Name:</label>
                <input type="text" id="profile-name" name="name" required />
            </div>
            
            <div class="form-group">
                <label for="profile-email">Email:</label>
                <input type="email" id="profile-email" name="email" required />
            </div>
            
            <div class="form-group">
                <label for="profile-phone">Phone:</label>
                <input type="tel" id="profile-phone" name="phone" />
            </div>
            
            <button type="submit" class="btn btn-primary">Save Changes</button>
            <button type="button" id="cancel-btn" class="btn btn-secondary">Cancel</button>
        </form>
    </div>
</div>

<script src="/resources/js/modal-manager.js"></script>
