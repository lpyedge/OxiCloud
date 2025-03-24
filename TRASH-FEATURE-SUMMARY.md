# Trash Feature Implementation Summary

This document summarizes the implementation of the trash/recycle bin feature in OxiCloud.

## Architecture Overview

The trash feature is implemented following the hexagonal architecture (clean architecture) principles of OxiCloud:

1. **Domain Layer** (`/src/domain/`):
   - Entities: `TrashedItem` representing files and folders in the trash bin
   - Repository interfaces: `TrashRepository` defining operations for trash management

2. **Application Layer** (`/src/application/`):
   - DTOs: `TrashedItemDto` for data transfer between layers
   - Ports: `TrashUseCase` defining the operations available to clients
   - Services: `TrashService` implementing the trash use cases

3. **Infrastructure Layer** (`/src/infrastructure/`):
   - Repositories: `TrashFsRepository` for filesystem-based trash storage
   - Extensions to existing repositories: `FileRepositoryTrash` and `FolderRepositoryTrash`
   - Services: `TrashCleanupService` for automatic cleanup of expired trash items

4. **Interface Layer** (`/src/interfaces/`):
   - API handlers: `trash_handler.rs` providing HTTP endpoints for trash operations
   - Routes: Updated `routes.rs` to include trash-related endpoints

## Key Features

1. **Soft Deletion**: Moving files and folders to trash instead of immediate permanent deletion
2. **Per-User Trash**: Each user has their own isolated trash bin
3. **Retention Policy**: Items are automatically deleted after a configurable time period
4. **Restoration**: Items can be restored to their original location
5. **Permanent Deletion**: Items can be permanently deleted before the retention period expires
6. **Empty Trash**: All items in the trash can be permanently deleted at once

## API Endpoints

The trash feature exposes the following REST API endpoints:

- `GET /api/trash`: List all items in the user's trash bin
- `DELETE /api/files/trash/:file_id`: Move a file to trash
- `DELETE /api/folders/trash/:folder_id`: Move a folder to trash
- `POST /api/trash/:trash_id/restore`: Restore an item from trash to its original location
- `DELETE /api/trash/:trash_id`: Permanently delete an item from trash
- `DELETE /api/trash/empty`: Empty the entire trash bin

## Testing

The trash feature includes comprehensive testing:

1. **Unit Tests**: Testing the `TrashService` application service
   - Test moving files and folders to trash
   - Test restoring items from trash
   - Test permanent deletion
   - Test empty trash operation

2. **Integration Tests**: Python script to test the API endpoints
   - End-to-end testing of all trash operations
   - Verification of proper behavior for moving, listing, restoring, and deleting

3. **Shell Script**: For manual testing and demonstration
   - Individual tests for each operation
   - Visual feedback of successful operations

## Configuration

The trash feature can be configured via environment variables:

- `TRASH_ENABLED`: Enable/disable the trash feature (default: true)
- `TRASH_RETENTION_DAYS`: Number of days to keep items in trash before automatic deletion (default: 30)

## Implementation Details

1. **Physical File Storage**: When items are moved to trash, they are physically moved to a `.trash` directory
2. **Metadata Storage**: Information about trashed items is stored in a separate database table or file
3. **User Isolation**: Trash items are isolated by user ID to prevent access to other users' trash
4. **Automatic Cleanup**: A background job runs periodically to clean up expired trash items
5. **Transaction Safety**: Operations are designed to be atomic and safe, with proper error handling

## Future Enhancements

Potential improvements for the trash feature:

1. **Trash Quotas**: Limit the amount of storage a user can use for trash
2. **Batch Operations**: Add support for trashing, restoring, or deleting multiple items at once
3. **Storage Optimization**: Implement deduplication for trashed items to save storage space
4. **Version Control**: Keep track of file versions when moving to trash
5. **Scheduled Cleanup**: Allow users to configure custom retention periods
6. **Trash Monitoring**: Add metrics and alerts for trash usage and cleanup operations