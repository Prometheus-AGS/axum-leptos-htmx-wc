# Spec Delta: Offline Mode

## ADDED Requirements

### Requirement: Offline Conversation Access

Users must be able to view past conversations even when the server is unreachable.

#### Scenario: Load Cached Assets

- **Given** the user is offline
- **When** they navigate to the application
- **Then** the Service Worker should serve cached static assets (HTML, JS, CSS).

#### Scenario: Access Local Data

- **Given** the user is offline
- **When** the application loads
- **Then** PGlite should initialize and allow the user to browse and read past conversations stored locally.
