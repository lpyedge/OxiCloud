pub mod file_fs_repository;
pub mod folder_fs_repository;
pub mod parallel_file_processor;

// Nuevos repositorios refactorizados
pub mod file_metadata_manager;
pub mod file_path_resolver;
pub mod file_fs_read_repository;
pub mod file_fs_write_repository;
pub mod trash_fs_repository;
pub mod file_fs_repository_trash;
pub mod folder_fs_repository_trash;
pub mod share_fs_repository;

// Repositorios PostgreSQL
pub mod pg;

// Re-exportar para facilitar acceso
pub use file_metadata_manager::FileMetadataManager;
pub use file_path_resolver::FilePathResolver;
pub use file_fs_read_repository::FileFsReadRepository;
pub use file_fs_write_repository::FileFsWriteRepository;
pub use pg::{UserPgRepository, SessionPgRepository};
pub use share_fs_repository::ShareFsRepository;