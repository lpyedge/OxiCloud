pub mod dtos;
pub mod ports;
pub mod services;
pub mod transactions;

// Re-exportaciones para facilitar el acceso a los principales puertos
pub use ports::inbound::FolderUseCase;
pub use ports::file_ports::{FileUploadUseCase, FileRetrievalUseCase, FileManagementUseCase, FileUseCaseFactory};
pub use ports::outbound::{FolderStoragePort, IdMappingPort};
pub use ports::storage_ports::{FileReadPort, FileWritePort, FilePathResolutionPort, StorageVerificationPort, DirectoryManagementPort};
