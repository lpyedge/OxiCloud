# OxiCloud TODO List

This document contains the task list for the development of OxiCloud, a minimalist and efficient cloud storage system similar to NextCloud but optimized for performance.

## Phase 1: Basic File Functionalities

### Folder System
- [x] Implement API for creating folders
- [x] Add support for hierarchical paths in the backend
- [ ] Update UI to show folder structure (tree)
- [x] Implement navigation between folders
- [x] Add functionality to rename folders
- [x] Add option to move files between folders

### File Preview
- [ ] Implement integrated image viewer
- [ ] Add basic PDF viewer
- [ ] Generate thumbnails for images
- [ ] Implement specific icons by file type
- [ ] Add text/code preview

### Enhanced Search
- [ ] Implement search by name
- [ ] Add filters by file type
- [ ] Implement search by date range
- [ ] Add filter by file size
- [ ] Add search within specific folders
- [ ] Implement cache for search results

### UI/UX Optimizations
- [ ] Improve responsive design for mobile devices
- [ ] Implement drag & drop between folders
- [ ] Add support for multiple file selection
- [x] Implement multiple file uploads
- [ ] Add progress indicators for long operations
- [ ] Implement UI notifications for events

## Phase 2: Authentication and Multi-User

### User System
- [ ] Design data model for users
- [ ] Implement user registration
- [ ] Create login system
- [ ] Add user profile page
- [ ] Implement password recovery
- [ ] Separate storage by user

### Quotas and Permissions
- [ ] Implement storage quota system
- [ ] Add basic role system (admin/user)
- [ ] Create admin panel
- [ ] Implement folder-level permissions
- [ ] Add storage usage monitoring

### Basic Security
- [x] Implement secure password hashing with Argon2
- [x] Add session management
- [x] Implement JWT authentication token
- [ ] Add CSRF protection
- [ ] Implement login attempt limits
- [ ] Create activity logging system

## Phase 3: Collaboration Features

### File Sharing
- [ ] Implement shared link generation
- [ ] Add permission configuration for links
- [ ] Implement password protection for links
- [ ] Add expiration dates for shared links
- [ ] Create page to manage all shared resources
- [ ] Implement sharing notifications

### Recycle Bin
- [x] Design model for storing deleted files
- [x] Implement soft deletion (move to trash)
- [x] Add functionality to restore files
- [x] Implement automatic purge by time
- [x] Add option to manually empty trash
- [x] Implement storage limits for trash

### Activity Log
- [ ] Create model for activity events
- [ ] Implement logging of CRUD operations
- [ ] Add logging of access and security events
- [ ] Create activity history page
- [ ] Implement filters for activity log
- [ ] Add log export

## Phase 4: API and Synchronization

### Complete REST API
- [ ] Design OpenAPI specification
- [x] Implement endpoints for file operations
- [x] Add endpoints for users and authentication
- [ ] Implement automatic documentation (Swagger)
- [ ] Create API token system
- [ ] Implement rate limiting
- [ ] Add API versioning

### WebDAV Support
- [ ] Implement basic WebDAV server
- [ ] Add authentication for WebDAV
- [ ] Implement PROPFIND operations
- [ ] Add support for locking
- [ ] Test compatibility with standard clients
- [ ] Optimize WebDAV performance

### Sync Client
- [ ] Design client architecture in Rust
- [ ] Implement unidirectional synchronization
- [ ] Add bidirectional synchronization
- [ ] Implement conflict detection
- [ ] Add configuration options
- [ ] Create minimal client version for Windows/macOS/Linux

## Phase 5: Advanced Features

### File Encryption
- [ ] Research and select encryption algorithms
- [ ] Implement at-rest encryption for files
- [ ] Add key management
- [ ] Implement encryption for shared files
- [ ] Create security documentation

### File Versioning
- [ ] Design version storage system
- [ ] Implement version history
- [ ] Add difference visualization
- [ ] Implement version restoration
- [ ] Add version retention policies

### Basic Applications
- [ ] Design plugin/app system
- [ ] Implement basic text viewer/editor
- [ ] Add simple notes application
- [ ] Implement basic calendar
- [ ] Create API for third-party applications

## Continuous Optimizations

### Backend
- [x] Implement file cache with Rust
- [x] Enable Link Time Optimization (LTO) for better performance
- [x] Optimize large file transmission
- [x] Add adaptive compression by file type
- [x] Implement asynchronous processing for heavy tasks
- [ ] Optimize database queries
- [ ] Implement scaling strategies

### Frontend
- [ ] Optimize initial asset loading
- [ ] Implement lazy loading for large lists
- [ ] Add local cache (localStorage/IndexedDB)
- [ ] Optimize UI rendering
- [ ] Implement intelligent prefetching
- [ ] Add basic offline support

### Storage
- [ ] Research deduplication options
- [ ] Implement block storage
- [x] Add transparent compression by file type
- [ ] Implement log rotation and archiving
- [ ] Create automated backup system
- [ ] Add support for distributed storage

## Infrastructure and Deployment

- [x] Create Docker configuration
- [ ] Implement CI/CD with GitHub Actions
- [x] Add automated tests
- [ ] Create installation documentation
- [ ] Implement monitoring and alerts
- [ ] Add automatic update system