# Spec Delta: Tauri Packaging

## MODIFIED Requirements

### Requirement: MCP Server Execution

The system must support executing MCP servers as Tauri sidecars when running in a bundled environment.

#### Scenario: Resolve Sidecar Path

- **Given** the application is running in a Tauri environment
- **And** an MCP server is configured with a command that matches a sidecar
- **When** the MCP server is started
- **Then** the system should resolve the absolute path to the sidecar binary
- **And** execute it with the provided arguments and environment variables.

#### Scenario: Fallback to System Command

- **Given** the application is NOT running in a Tauri environment
- **When** an MCP server is started
- **Then** the system should execute the command as a standard system process.
