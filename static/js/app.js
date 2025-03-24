/**
 * OxiCloud - Main Application
 * This file contains the core functionality, initialization and state management
 */

// Global state
const app = {
    currentView: 'grid',   // Current view mode: 'grid' or 'list'
    currentPath: '',       // Current folder path
    currentFolder: null,   // Current folder object
    contextMenuTargetFolder: null,  // Target folder for context menu
    contextMenuTargetFile: null,    // Target file for context menu
    selectedTargetFolderId: "",     // Selected target folder for move operations
    moveDialogMode: 'file',         // Move dialog mode: 'file' or 'folder'
    isTrashView: false,    // Whether we're in trash view
    currentSection: 'files', // Current section: 'files' or 'trash'
};

// DOM elements
const elements = {
    // Will be populated on initialization
};

/**
 * Initialize the application
 */
function initApp() {
    // Cache DOM elements
    cacheElements();
    
    // Create menus and dialogs
    ui.initializeContextMenus();
    
    // Setup event listeners
    setupEventListeners();
    
    // Load initial view
    app.currentPath = '';
    ui.updateBreadcrumb('');
    loadFiles();
    
    // Initialize file renderer if available
    if (window.fileRenderer) {
        console.log('Using optimized file renderer');
    } else {
        console.log('Using standard file rendering');
    }
    
    // Check authentication
    checkAuthentication();
}

/**
 * Cache DOM elements for faster access
 */
function cacheElements() {
    elements.uploadBtn = document.getElementById('upload-btn');
    elements.dropzone = document.getElementById('dropzone');
    elements.fileInput = document.getElementById('file-input');
    elements.filesGrid = document.getElementById('files-grid');
    elements.filesListView = document.getElementById('files-list-view');
    elements.newFolderBtn = document.getElementById('new-folder-btn');
    elements.gridViewBtn = document.getElementById('grid-view-btn');
    elements.listViewBtn = document.getElementById('list-view-btn');
    elements.breadcrumb = document.querySelector('.breadcrumb');
    elements.logoutBtn = document.getElementById('logout-btn');
    elements.pageTitle = document.querySelector('.page-title');
    elements.actionsBar = document.querySelector('.actions-bar');
    elements.navItems = document.querySelectorAll('.nav-item');
    elements.trashBtn = document.querySelector('.nav-item:nth-child(5)'); // The trash nav item
}

/**
 * Setup event listeners for main UI elements
 */
function setupEventListeners() {
    // Set up drag and drop
    ui.setupDragAndDrop();
    
    // Upload button
    elements.uploadBtn.addEventListener('click', () => {
        elements.dropzone.style.display = elements.dropzone.style.display === 'none' ? 'block' : 'none';
        if (elements.dropzone.style.display === 'block') {
            elements.fileInput.click();
        }
    });
    
    // File input
    elements.fileInput.addEventListener('change', (e) => {
        if (e.target.files.length > 0) {
            fileOps.uploadFiles(e.target.files);
        }
    });
    
    // New folder button
    elements.newFolderBtn.addEventListener('click', () => {
        const folderName = prompt(window.i18n ? window.i18n.t('dialogs.new_name') : 'Nombre de la carpeta:');
        if (folderName) {
            fileOps.createFolder(folderName);
        }
    });
    
    // View toggle
    elements.gridViewBtn.addEventListener('click', ui.switchToGridView);
    elements.listViewBtn.addEventListener('click', ui.switchToListView);
    
    // Sidebar navigation
    elements.navItems.forEach(item => {
        item.addEventListener('click', () => {
            // Remove active class from all nav items
            elements.navItems.forEach(navItem => navItem.classList.remove('active'));
            
            // Add active class to clicked item
            item.classList.add('active');
            
            // Check if this is the trash item
            if (item === elements.trashBtn) {
                // Show trash view
                app.isTrashView = true;
                app.currentSection = 'trash';
                
                // Update UI
                elements.pageTitle.textContent = window.i18n ? window.i18n.t('nav.trash') : 'Papelera';
                elements.actionsBar.innerHTML = `
                    <div class="action-buttons">
                        <button class="btn btn-danger" id="empty-trash-btn">
                            <i class="fas fa-trash" style="margin-right: 5px;"></i> 
                            <span>${window.i18n ? window.i18n.t('trash.empty_trash') : 'Vaciar papelera'}</span>
                        </button>
                    </div>
                `;
                
                // Add event listener to empty trash button
                document.getElementById('empty-trash-btn').addEventListener('click', async () => {
                    if (await fileOps.emptyTrash()) {
                        loadTrashItems();
                    }
                });
                
                // Load trash items
                loadTrashItems();
            } else {
                // Show regular files view
                app.isTrashView = false;
                app.currentSection = 'files';
                
                // Reset UI
                elements.pageTitle.textContent = window.i18n ? window.i18n.t('nav.files') : 'Archivos';
                elements.actionsBar.innerHTML = `
                    <div class="action-buttons">
                        <button class="btn btn-primary" id="upload-btn">
                            <i class="fas fa-upload" style="margin-right: 5px;"></i> <span data-i18n="actions.upload">Subir</span>
                        </button>
                        <button class="btn btn-secondary" id="new-folder-btn">
                            <i class="fas fa-folder-plus" style="margin-right: 5px;"></i> <span data-i18n="actions.new_folder">Nueva carpeta</span>
                        </button>
                    </div>
                    <div class="view-toggle">
                        <button class="toggle-btn active" id="grid-view-btn" title="Vista de cuadrícula">
                            <i class="fas fa-th"></i>
                        </button>
                        <button class="toggle-btn" id="list-view-btn" title="Vista de lista">
                            <i class="fas fa-list"></i>
                        </button>
                    </div>
                `;
                
                // Restore event listeners
                document.getElementById('upload-btn').addEventListener('click', () => {
                    elements.dropzone.style.display = elements.dropzone.style.display === 'none' ? 'block' : 'none';
                    if (elements.dropzone.style.display === 'block') {
                        elements.fileInput.click();
                    }
                });
                
                document.getElementById('new-folder-btn').addEventListener('click', () => {
                    const folderName = prompt(window.i18n ? window.i18n.t('dialogs.new_name') : 'Nombre de la carpeta:');
                    if (folderName) {
                        fileOps.createFolder(folderName);
                    }
                });
                
                document.getElementById('grid-view-btn').addEventListener('click', ui.switchToGridView);
                document.getElementById('list-view-btn').addEventListener('click', ui.switchToListView);
                
                // Restore cached elements
                elements.uploadBtn = document.getElementById('upload-btn');
                elements.newFolderBtn = document.getElementById('new-folder-btn');
                elements.gridViewBtn = document.getElementById('grid-view-btn');
                elements.listViewBtn = document.getElementById('list-view-btn');
                
                // Load regular files
                app.currentPath = '';
                ui.updateBreadcrumb('');
                loadFiles();
            }
        });
    });
    
    // Load saved view preference
    const savedView = localStorage.getItem('oxicloud-view');
    if (savedView === 'list') {
        ui.switchToListView();
    }
    
    // Logout button
    elements.logoutBtn.addEventListener('click', logout);
    
    // Global events to close context menus
    document.addEventListener('click', (e) => {
        const folderMenu = document.getElementById('folder-context-menu');
        const fileMenu = document.getElementById('file-context-menu');
        
        if (folderMenu && folderMenu.style.display === 'block' && 
            !folderMenu.contains(e.target)) {
            ui.closeContextMenu();
        }
        
        if (fileMenu && fileMenu.style.display === 'block' && 
            !fileMenu.contains(e.target)) {
            ui.closeFileContextMenu();
        }
    });
}

/**
 * Load files and folders for the current path
 */
async function loadFiles() {
    try {
        let url = '/api/folders';
        if (app.currentPath) {
            // Use the correct endpoint for folder contents
            url = `/api/folders/${app.currentPath}/contents`;
        }
        
        const response = await fetch(url);
        if (!response.ok) {
            throw new Error(`Server responded with status: ${response.status}`);
        }
        const folders = await response.json();
        
        // Clear existing files in both views
        elements.filesGrid.innerHTML = '';
        elements.filesListView.innerHTML = `
            <div class="list-header">
                <div data-i18n="files.name">Nombre</div>
                <div data-i18n="files.type">Tipo</div>
                <div data-i18n="files.size">Tamaño</div>
                <div data-i18n="files.modified">Modificado</div>
            </div>
        `;
        
        // Translate the header if i18n is available
        if (window.i18n && window.i18n.translatePage) {
            window.i18n.translatePage();
        }
        
        // Add folders (check if it's an array)
        const folderList = Array.isArray(folders) ? folders : [];
        folderList.forEach(folder => {
            ui.addFolderToView(folder);
        });
        
        // Also load files in this folder
        let filesUrl = '/api/files';
        if (app.currentPath) {
            filesUrl += `?folder_id=${app.currentPath}`;
        }
        
        try {
            console.log(`Fetching files from: ${filesUrl}`);
            const filesResponse = await fetch(filesUrl);
            console.log(`Files response status: ${filesResponse.status}`);
            
            if (filesResponse.ok) {
                const files = await filesResponse.json();
                console.log(`Files received:`, files);
                
                // Add files (check if it's an array)
                const fileList = Array.isArray(files) ? files : [];
                console.log(`Processing ${fileList.length} files`);
                
                fileList.forEach(file => {
                    console.log(`Adding file to view: ${file.name} (${file.id})`);
                    ui.addFileToView(file);
                });
            } else {
                const errorText = await filesResponse.text();
                console.error(`Error loading files: ${filesResponse.status} - ${errorText}`);
            }
        } catch (error) {
            console.error('Error loading files:', error);
            // File API may not be implemented yet, so we silently ignore this error
        }
        
        // Update file icons based on file type
        ui.updateFileIcons();
    } catch (error) {
        console.error('Error loading folders:', error);
        ui.showNotification('Error', 'Could not load files and folders');
    }
}

/**
 * Format file size in human-readable format
 * @param {number} bytes - Size in bytes
 * @return {string} Formatted size
 */
function formatFileSize(bytes) {
    if (bytes === 0) return '0 Bytes';
    
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
}

/**
 * Load trash items 
 */
async function loadTrashItems() {
    try {
        // Clear existing content
        elements.filesGrid.innerHTML = '';
        elements.filesListView.innerHTML = `
            <div class="list-header">
                <div data-i18n="files.name">Nombre</div>
                <div data-i18n="files.type">Tipo</div>
                <div data-i18n="files.original_location">Ubicación original</div>
                <div data-i18n="files.deleted_date">Fecha eliminación</div>
                <div data-i18n="files.actions">Acciones</div>
            </div>
        `;
        
        // Update breadcrumb for trash
        ui.updateBreadcrumb(window.i18n ? window.i18n.t('nav.trash') : 'Papelera');
        
        // Get trash items
        const trashItems = await fileOps.getTrashItems();
        
        if (trashItems.length === 0) {
            // Show empty state
            const emptyState = document.createElement('div');
            emptyState.className = 'empty-state';
            emptyState.innerHTML = `
                <i class="fas fa-trash" style="font-size: 48px; color: #ddd; margin-bottom: 16px;"></i>
                <p>${window.i18n ? window.i18n.t('trash.empty_state') : 'La papelera está vacía'}</p>
            `;
            elements.filesGrid.appendChild(emptyState);
            return;
        }
        
        // Process each trash item
        trashItems.forEach(item => {
            addTrashItemToView(item);
        });
        
    } catch (error) {
        console.error('Error loading trash items:', error);
        window.ui.showNotification('Error', 'Error al cargar elementos de la papelera');
    }
}

/**
 * Add a trash item to the view
 * @param {Object} item - Trash item object
 */
function addTrashItemToView(item) {
    const isFile = item.item_type === 'file';
    const iconClass = isFile ? 'fas fa-file' : 'fas fa-folder';
    
    // Format date
    const deletedDate = new Date(item.deleted_at * 1000);
    const formattedDate = deletedDate.toLocaleDateString() + ' ' +
                         deletedDate.toLocaleTimeString([], {hour: '2-digit', minute:'2-digit'});
                         
    // Item type label
    const typeLabel = isFile ? 
        (window.i18n ? window.i18n.t('files.file_types.file') : 'Archivo') :
        (window.i18n ? window.i18n.t('files.file_types.folder') : 'Carpeta');
    
    // Grid view element
    const gridElement = document.createElement('div');
    gridElement.className = 'file-card trash-item';
    gridElement.dataset.trashId = item.id;
    gridElement.dataset.originalId = item.original_id;
    gridElement.dataset.itemType = item.item_type;
    gridElement.innerHTML = `
        <div class="file-icon">
            <i class="${iconClass}"></i>
        </div>
        <div class="file-name">${item.name}</div>
        <div class="file-info">${typeLabel} - ${formattedDate}</div>
        <div class="trash-actions">
            <button class="btn-restore" title="Restaurar">
                <i class="fas fa-undo"></i>
            </button>
            <button class="btn-delete" title="Eliminar permanentemente">
                <i class="fas fa-trash"></i>
            </button>
        </div>
    `;
    
    // Add action buttons event listeners
    gridElement.querySelector('.btn-restore').addEventListener('click', async (e) => {
        e.stopPropagation();
        if (await fileOps.restoreFromTrash(item.id)) {
            loadTrashItems();
        }
    });
    
    gridElement.querySelector('.btn-delete').addEventListener('click', async (e) => {
        e.stopPropagation();
        if (await fileOps.deletePermanently(item.id)) {
            loadTrashItems();
        }
    });
    
    elements.filesGrid.appendChild(gridElement);
    
    // List view element
    const listElement = document.createElement('div');
    listElement.className = 'file-item trash-item';
    listElement.dataset.trashId = item.id;
    listElement.dataset.originalId = item.original_id;
    listElement.dataset.itemType = item.item_type;
    
    listElement.innerHTML = `
        <div class="name-cell">
            <div class="file-icon">
                <i class="${iconClass}"></i>
            </div>
            <span>${item.name}</span>
        </div>
        <div class="type-cell">${typeLabel}</div>
        <div class="path-cell">${item.original_path || '--'}</div>
        <div class="date-cell">${formattedDate}</div>
        <div class="actions-cell">
            <button class="btn-restore" title="Restaurar">
                <i class="fas fa-undo"></i>
            </button>
            <button class="btn-delete" title="Eliminar permanentemente">
                <i class="fas fa-trash"></i>
            </button>
        </div>
    `;
    
    // Add action buttons event listeners for list view
    listElement.querySelector('.btn-restore').addEventListener('click', async (e) => {
        e.stopPropagation();
        if (await fileOps.restoreFromTrash(item.id)) {
            loadTrashItems();
        }
    });
    
    listElement.querySelector('.btn-delete').addEventListener('click', async (e) => {
        e.stopPropagation();
        if (await fileOps.deletePermanently(item.id)) {
            loadTrashItems();
        }
    });
    
    elements.filesListView.appendChild(listElement);
}

// Expose needed functions to global scope
window.app = app;
window.loadFiles = loadFiles;
window.loadTrashItems = loadTrashItems;
window.formatFileSize = formatFileSize;

// Set up global selectFolder function for navigation
window.selectFolder = (id, name) => {
    app.currentPath = id;
    ui.updateBreadcrumb(name);
    loadFiles();
};

/**
 * Check if user is authenticated
 */
function checkAuthentication() {
    // Nombres de variables según auth.js
    const TOKEN_KEY = 'oxicloud_token';
    const TOKEN_EXPIRY_KEY = 'oxicloud_token_expiry';
    const USER_DATA_KEY = 'oxicloud_user';
    
    const token = localStorage.getItem(TOKEN_KEY);
    const tokenExpiry = localStorage.getItem(TOKEN_EXPIRY_KEY);
    
    if (!token || !tokenExpiry || new Date(tokenExpiry) < new Date()) {
        // No token or expired token
        window.location.href = '/login';
        return;
    }
    
    // Display user information if available
    const userData = JSON.parse(localStorage.getItem(USER_DATA_KEY) || '{}');
    if (userData.username) {
        // Update user avatar with initials
        const userInitials = userData.username.substring(0, 2).toUpperCase();
        const userAvatar = document.querySelector('.user-avatar');
        if (userAvatar) {
            userAvatar.textContent = userInitials;
        }
    }
}

/**
 * Logout - clear all auth data and redirect to login
 */
function logout() {
    // Nombres de variables según auth.js
    const TOKEN_KEY = 'oxicloud_token';
    const REFRESH_TOKEN_KEY = 'oxicloud_refresh_token';
    const TOKEN_EXPIRY_KEY = 'oxicloud_token_expiry';
    const USER_DATA_KEY = 'oxicloud_user';
    
    // Clear all authentication data
    localStorage.removeItem(TOKEN_KEY);
    localStorage.removeItem(REFRESH_TOKEN_KEY);
    localStorage.removeItem(TOKEN_EXPIRY_KEY);
    localStorage.removeItem(USER_DATA_KEY);
    
    // Redirect to login page
    window.location.href = '/login';
}

// Initialize app when DOM is ready
document.addEventListener('DOMContentLoaded', initApp);
