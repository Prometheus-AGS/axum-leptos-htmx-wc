# axum-leptos-htmx-wc TypeScript SDK

TypeScript SDK for axum-leptos-htmx-wc.

## Installation

```bash
npm install @prometheus-ags/axum-leptos-htmx-wc-sdk
# or
bun add @prometheus-ags/axum-leptos-htmx-wc-sdk
```

## Usage

```typescript
import { Client } from '@prometheus-ags/axum-leptos-htmx-wc-sdk';

const client = new Client('http://localhost:3000');

// Chat API
const chat = await client.chat.send('Hello!');
console.log('Session:', chat.session_id);

// Knowledge Base API
const kbs = await client.knowledge.list();
for (const kb of kbs) {
  console.log(`KB: ${kb.name} (${kb.id})`);
}

// Search
const results = await client.knowledge.search('kb-id', 'query');
for (const result of results.results) {
  console.log(`Score: ${result.score.toFixed(2)} - ${result.content}`);
}

// SSE Streaming
const eventSource = client.runs.stream('run-id');
eventSource.onmessage = (event) => {
  console.log('Event:', event.data);
};
```

## License

MIT
