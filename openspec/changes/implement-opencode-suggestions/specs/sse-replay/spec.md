# Spec Delta: SSE Replay

## ADDED Requirements

### Requirement: SSE Reconnection Reliability

The system must support seamless reconnection for SSE streams using `Last-Event-ID`.

#### Scenario: Reconnect with Last-Event-ID

- **Given** an SSE connection is interrupted
- **When** the client reconnects
- **Then** it should include the `Last-Event-ID` header.
- **And** the server should replay missed events starting from that ID.
