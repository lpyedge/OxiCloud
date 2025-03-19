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
    
    // Load saved view preference
    const savedView = localStorage.getItem('oxicloud-view');
    if (savedView === 'list') {
        ui.switchToListView();
    }
    
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
            url += `/${app.currentPath}`;
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
            const filesResponse = await fetch(filesUrl);
            if (filesResponse.ok) {
                const files = await filesResponse.json();
                
                // Add files (check if it's an array)
                const fileList = Array.isArray(files) ? files : [];
                fileList.forEach(file => {
                    ui.addFileToView(file);
                });
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

// Expose needed functions to global scope
window.app = app;
window.loadFiles = loadFiles;
window.formatFileSize = formatFileSize;

// Set up global selectFolder function for navigation
window.selectFolder = (id, name) => {
    app.currentPath = id;
    ui.updateBreadcrumb(name);
    loadFiles();
};

// Initialize app when DOM is ready
document.addEventListener('DOMContentLoaded', initApp);
