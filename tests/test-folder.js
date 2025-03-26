// Función para crear carpeta
async function createFolder() {
    try {
        const response = await fetch('http://localhost:8085/api/folders', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify({
                name: 'test_folder_js',
                parent_id: null
            })
        });
        
        const data = await response.json();
        console.log('Respuesta de creación de carpeta:', data);
        
        return data;
    } catch (error) {
        console.error('Error al crear carpeta:', error);
        return null;
    }
}

// Función para listar carpetas
async function listFolders() {
    try {
        const response = await fetch('http://localhost:8085/api/folders');
        const data = await response.json();
        console.log('Listado de carpetas:', data);
        
        return data;
    } catch (error) {
        console.error('Error al listar carpetas:', error);
        return [];
    }
}

// Ejecutar las funciones
async function runTest() {
    console.log('Creando carpeta nueva...');
    await createFolder();
    
    console.log('Listando carpetas...');
    await listFolders();
}

runTest();