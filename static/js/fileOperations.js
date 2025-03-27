/**
 * OxiCloud - File Operations Module
 * This file handles file and folder operations (create, move, delete, rename, upload)
 */

// File Operations Module
const fileOps = {
    /**
     * Upload files to the server
     * @param {FileList} files - Files to upload
     */
    async uploadFiles(files) {
        const progressBar = document.querySelector('.progress-fill');
        const uploadProgressDiv = document.querySelector('.upload-progress');
        uploadProgressDiv.style.display = 'block';
        progressBar.style.width = '0%';

        let uploadedCount = 0;
        const totalFiles = files.length;

        for (let i = 0; i < totalFiles; i++) {
            const file = files[i];
            const formData = new FormData();
            formData.append('file', file);

            if (window.app.currentPath) {
                formData.append('folder_id', window.app.currentPath);
            }

            try {
                console.log(`Uploading file to current path: ${window.app.currentPath || 'root'}`);
                
                // Usamos la URL correcta para la subida de archivos
                console.log('Formulario a enviar:', {
                    file: file.name,
                    size: file.size,
                    folder_id: window.app.currentPath || 'root'
                });
                
                const response = await fetch('/api/files/upload', {
                    method: 'POST',
                    body: formData
                });
                
                console.log('Respuesta del servidor:', {
                    status: response.status,
                    statusText: response.statusText
                });

                // Update progress
                uploadedCount++;
                const percentComplete = (uploadedCount / totalFiles) * 100;
                progressBar.style.width = percentComplete + '%';

                if (response.ok) {
                    const responseData = await response.json();
                    console.log(`Successfully uploaded ${file.name}`, responseData);

                    if (i === totalFiles - 1) {
                        // Last file uploaded
                        console.log('Recargando lista de archivos después de subida');
                        window.loadFiles();
                        setTimeout(() => {
                            document.getElementById('dropzone').style.display = 'none';
                            uploadProgressDiv.style.display = 'none';
                        }, 1000);

                        // Show success notification
                        window.ui.showNotification('Archivo subido', `${file.name} completado`);
                    }
                } else {
                    const errorData = await response.text();
                    console.error('Upload error:', errorData);
                    window.ui.showNotification('Error', `Error al subir el archivo: ${file.name}`);
                }
            } catch (error) {
                console.error('Network error during upload:', error);
                window.ui.showNotification('Error', `Error de red al subir el archivo: ${file.name}`);
            }
        }
    },

    /**
     * Create a new folder
     * @param {string} name - Folder name
     */
    async createFolder(name) {
        try {
            const response = await fetch('/api/folders', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({
                    name: name,
                    parent_id: window.app.currentPath || null
                })
            });

            if (response.ok) {
                window.loadFiles();
                window.ui.showNotification('Carpeta creada', `"${name}" creada correctamente`);
            } else {
                const errorData = await response.text();
                console.error('Create folder error:', errorData);
                window.ui.showNotification('Error', 'Error al crear la carpeta');
            }
        } catch (error) {
            console.error('Error creating folder:', error);
            window.ui.showNotification('Error', 'Error al crear la carpeta');
        }
    },

    /**
     * Move a file to another folder
     * @param {string} fileId - File ID
     * @param {string} targetFolderId - Target folder ID
     * @returns {Promise<boolean>} - Success status
     */
    async moveFile(fileId, targetFolderId) {
        try {
            const response = await fetch(`/api/files/${fileId}/move`, {
                method: 'PUT',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({
                    folder_id: targetFolderId === "" ? null : targetFolderId
                })
            });

            if (response.ok) {
                // Reload files after moving
                await window.loadFiles();
                window.ui.showNotification('Archivo movido', 'Archivo movido correctamente');
                return true;
            } else {
                let errorMessage = 'Error desconocido';
                try {
                    const errorData = await response.json();
                    errorMessage = errorData.error || 'Error desconocido';
                } catch (e) {
                    errorMessage = 'Error al procesar la respuesta del servidor';
                }
                window.ui.showNotification('Error', `Error al mover el archivo: ${errorMessage}`);
                return false;
            }
        } catch (error) {
            console.error('Error moving file:', error);
            window.ui.showNotification('Error', 'Error al mover el archivo');
            return false;
        }
    },

    /**
     * Move a folder to another folder
     * @param {string} folderId - Folder ID
     * @param {string} targetFolderId - Target folder ID
     * @returns {Promise<boolean>} - Success status
     */
    async moveFolder(folderId, targetFolderId) {
        try {
            const response = await fetch(`/api/folders/${folderId}/move`, {
                method: 'PUT',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({
                    parent_id: targetFolderId === "" ? null : targetFolderId
                })
            });

            if (response.ok) {
                // Reload files after moving
                await window.loadFiles();
                window.ui.showNotification('Carpeta movida', 'Carpeta movida correctamente');
                return true;
            } else {
                let errorMessage = 'Error desconocido';
                try {
                    const errorData = await response.json();
                    errorMessage = errorData.error || 'Error desconocido';
                } catch (e) {
                    errorMessage = 'Error al procesar la respuesta del servidor';
                }
                window.ui.showNotification('Error', `Error al mover la carpeta: ${errorMessage}`);
                return false;
            }
        } catch (error) {
            console.error('Error moving folder:', error);
            window.ui.showNotification('Error', 'Error al mover la carpeta');
            return false;
        }
    },

    /**
     * Rename a folder
     * @param {string} folderId - Folder ID
     * @param {string} newName - New folder name
     * @returns {Promise<boolean>} - Success status
     */
    async renameFolder(folderId, newName) {
        try {
            console.log(`Renaming folder ${folderId} to "${newName}"`);

            const response = await fetch(`/api/folders/${folderId}/rename`, {
                method: 'PUT',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({ name: newName })
            });

            console.log('Response status:', response.status);

            if (response.ok) {
                window.ui.showNotification('Carpeta renombrada', `Carpeta renombrada a "${newName}"`);
                return true;
            } else {
                const errorText = await response.text();
                console.error('Error response:', errorText);

                let errorMessage = 'Error desconocido';
                try {
                    // Try to parse as JSON
                    const errorData = JSON.parse(errorText);
                    errorMessage = errorData.error || response.statusText;
                } catch (e) {
                    // If not JSON, use text as is
                    errorMessage = errorText || response.statusText;
                }

                window.ui.showNotification('Error', `Error al renombrar la carpeta: ${errorMessage}`);
                return false;
            }
        } catch (error) {
            console.error('Error renaming folder:', error);
            window.ui.showNotification('Error', 'Error al renombrar la carpeta');
            return false;
        }
    },

    /**
     * Move a file to trash
     * @param {string} fileId - File ID
     * @param {string} fileName - File name
     * @returns {Promise<boolean>} - Success status
     */
    async deleteFile(fileId, fileName) {
        if (!confirm(`¿Estás seguro de que quieres mover a la papelera el archivo "${fileName}"?`)) {
            return false;
        }
        
        try {
            // Use the trash API endpoint
            const response = await fetch(`/api/trash/files/${fileId}`, {
                method: 'DELETE'
            });

            if (response.ok) {
                window.loadFiles();
                window.ui.showNotification('Archivo movido a papelera', `"${fileName}" movido a la papelera`);
                return true;
            } else {
                // Fallback to direct deletion if trash fails
                const fallbackResponse = await fetch(`/api/files/${fileId}`, {
                    method: 'DELETE'
                });
                
                if (fallbackResponse.ok) {
                    window.loadFiles();
                    window.ui.showNotification('Archivo eliminado', `"${fileName}" eliminado correctamente`);
                    return true;
                } else {
                    window.ui.showNotification('Error', 'Error al eliminar el archivo');
                    return false;
                }
            }
        } catch (error) {
            console.error('Error deleting file:', error);
            window.ui.showNotification('Error', 'Error al eliminar el archivo');
            return false;
        }
    },

    /**
     * Move a folder to trash
     * @param {string} folderId - Folder ID
     * @param {string} folderName - Folder name
     * @returns {Promise<boolean>} - Success status
     */
    async deleteFolder(folderId, folderName) {
        if (!confirm(`¿Estás seguro de que quieres mover a la papelera la carpeta "${folderName}" y todo su contenido?`)) {
            return false;
        }
        
        try {
            // Use the trash API endpoint
            const response = await fetch(`/api/trash/folders/${folderId}`, {
                method: 'DELETE'
            });

            if (response.ok) {
                // If we're inside the folder we just deleted, go back up
                if (window.app.currentPath === folderId) {
                    window.app.currentPath = '';
                    window.ui.updateBreadcrumb('');
                }
                window.loadFiles();
                window.ui.showNotification('Carpeta movida a papelera', `"${folderName}" movida a la papelera`);
                return true;
            } else {
                // Fallback to direct deletion if trash fails
                const fallbackResponse = await fetch(`/api/folders/${folderId}`, {
                    method: 'DELETE'
                });
                
                if (fallbackResponse.ok) {
                    // If we're inside the folder we just deleted, go back up
                    if (window.app.currentPath === folderId) {
                        window.app.currentPath = '';
                        window.ui.updateBreadcrumb('');
                    }
                    window.loadFiles();
                    window.ui.showNotification('Carpeta eliminada', `"${folderName}" eliminada correctamente`);
                    return true;
                } else {
                    window.ui.showNotification('Error', 'Error al eliminar la carpeta');
                    return false;
                }
            }
        } catch (error) {
            console.error('Error deleting folder:', error);
            window.ui.showNotification('Error', 'Error al eliminar la carpeta');
            return false;
        }
    },
    
    /**
     * Obtener elementos de la papelera
     * @returns {Promise<Array>} - Lista de elementos en la papelera
     */
    async getTrashItems() {
        try {
            const response = await fetch('/api/trash');
            
            if (response.ok) {
                return await response.json();
            } else {
                console.error('Error fetching trash items:', response.statusText);
                return [];
            }
        } catch (error) {
            console.error('Error fetching trash items:', error);
            return [];
        }
    },
    
    /**
     * Restaurar un elemento desde la papelera
     * @param {string} trashId - ID del elemento en la papelera
     * @returns {Promise<boolean>} - Éxito de la operación
     */
    async restoreFromTrash(trashId) {
        try {
            const response = await fetch(`/api/trash/${trashId}/restore`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({})
            });
            
            if (response.ok) {
                window.ui.showNotification('Elemento restaurado', 'Elemento restaurado correctamente');
                return true;
            } else {
                window.ui.showNotification('Error', 'Error al restaurar el elemento');
                return false;
            }
        } catch (error) {
            console.error('Error restoring item from trash:', error);
            window.ui.showNotification('Error', 'Error al restaurar el elemento');
            return false;
        }
    },
    
    /**
     * Eliminar permanentemente un elemento de la papelera
     * @param {string} trashId - ID del elemento en la papelera
     * @returns {Promise<boolean>} - Éxito de la operación
     */
    async deletePermanently(trashId) {
        if (!confirm('¿Estás seguro de que quieres eliminar permanentemente este elemento? Esta acción no se puede deshacer.')) {
            return false;
        }
        
        try {
            const response = await fetch(`/api/trash/${trashId}`, {
                method: 'DELETE'
            });
            
            if (response.ok) {
                window.ui.showNotification('Elemento eliminado', 'Elemento eliminado permanentemente');
                return true;
            } else {
                window.ui.showNotification('Error', 'Error al eliminar el elemento');
                return false;
            }
        } catch (error) {
            console.error('Error deleting item permanently:', error);
            window.ui.showNotification('Error', 'Error al eliminar el elemento');
            return false;
        }
    },
    
    /**
     * Vaciar la papelera
     * @returns {Promise<boolean>} - Éxito de la operación
     */
    async emptyTrash() {
        const confirmMsg = window.i18n ? window.i18n.t('trash.empty_confirm') : '¿Estás seguro de que quieres vaciar la papelera? Esta acción eliminará permanentemente todos los elementos.';
        if (!confirm(confirmMsg)) {
            return false;
        }
        
        try {
            const response = await fetch('/api/trash/empty', {
                method: 'DELETE'
            });
            
            if (response.ok) {
                window.ui.showNotification('Papelera vaciada', 'La papelera ha sido vaciada correctamente');
                return true;
            } else {
                window.ui.showNotification('Error', 'Error al vaciar la papelera');
                return false;
            }
        } catch (error) {
            console.error('Error emptying trash:', error);
            window.ui.showNotification('Error', 'Error al vaciar la papelera');
            return false;
        }
    }
};

// Expose file operations module globally
window.fileOps = fileOps;
