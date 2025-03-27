pub mod batch_operations;
pub mod file_service;
pub mod folder_service;
pub mod i18n_application_service;
pub mod storage_mediator;

// Nuevos servicios refactorizados
pub mod file_upload_service;
pub mod file_retrieval_service;
pub mod file_management_service;
pub mod file_use_case_factory;
pub mod auth_application_service;
pub mod trash_service;
pub mod search_service;

#[cfg(test)]
mod trash_service_test;

// Re-exportar para facilitar acceso
pub use file_upload_service::FileUploadService;
pub use file_retrieval_service::FileRetrievalService;
pub use file_management_service::FileManagementService;
pub use file_use_case_factory::AppFileUseCaseFactory;
