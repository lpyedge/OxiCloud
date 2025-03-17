/**
 * Language Selector Component for OxiCloud
 */

// Language codes and names
const languages = [
    { code: 'en', name: 'English' },
    { code: 'es', name: 'Español' }
];

/**
 * Creates and initializes a language selector component
 * @param {string} containerId - ID of the container element
 */
function createLanguageSelector(containerId = 'language-selector') {
    // Get or create container
    let container = document.getElementById(containerId);
    if (!container) {
        console.warn(`Container with ID "${containerId}" not found, creating one.`);
        container = document.createElement('div');
        container.id = containerId;
        document.body.appendChild(container);
    }
    
    // Create dropdown
    const select = document.createElement('select');
    select.className = 'language-select';
    select.setAttribute('aria-label', 'Select language');
    
    // Add options
    languages.forEach(lang => {
        const option = document.createElement('option');
        option.value = lang.code;
        option.textContent = lang.name;
        select.appendChild(option);
    });
    
    // Set current language
    const currentLocale = window.i18n ? window.i18n.getCurrentLocale() : 'en';
    select.value = currentLocale;
    
    // Add change event
    select.addEventListener('change', async (e) => {
        const locale = e.target.value;
        if (window.i18n) {
            await window.i18n.setLocale(locale);
        }
    });
    
    // Add to container
    container.innerHTML = '';
    container.appendChild(select);
    
    // Add event listener for locale changes
    window.addEventListener('localeChanged', (e) => {
        select.value = e.detail.locale;
    });
    
    return container;
}

// Create language selector when DOM is ready
document.addEventListener('DOMContentLoaded', () => {
    // Create language selector
    createLanguageSelector();
});