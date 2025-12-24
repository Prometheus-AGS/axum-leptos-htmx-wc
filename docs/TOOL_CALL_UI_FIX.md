# Tool Call UI Fixes

## Issues Fixed

### 1. Tool Call Data Not Showing ✅
**Problem:** First tool call showed no data in arguments section.

**Root Cause:** Attribute name mismatch between `chat-stream` renderer and `chat-tool-call` component:
- Renderer was using: `tool-id`, `tool-name`
- Component expected: `call-id`, `name`
- Renderer was passing: `argumentsJson` (wrong property name)
- Component expected: `arguments`
- Status was not being passed at all

**Fix:**
```typescript
// web/components/chat-stream/chat-stream.ts
private renderToolCall(item: ToolCallItem): string {
  return `
    <chat-tool-call 
      call-id="${item.id}"              // Fixed: was tool-id
      name="${escapeHtml(item.name)}"   // Fixed: was tool-name
      arguments='${escapeHtml(item.argumentsRaw)}'  // Fixed: was argumentsJson
      status="${item.status}"           // Added: was missing
      result='${hasResult ? escapeHtml(JSON.stringify(item.result)) : ''}'
      has-result="${hasResult}">
    </chat-tool-call>
  `;
}
```

### 2. "Streaming..." Status Never Updates ✅
**Problem:** Tool call blocks always showed "Streaming..." even after completion.

**Root Cause:** The `status` attribute was not being passed from the renderer to the component.

**Fix:** Added `status="${item.status}"` to the component attributes, which now properly reflects:
- `"streaming"` - While tool call is in progress (animated pulse)
- `"complete"` - When tool call finishes
- Result badge - When result is available (✓ Success / ✗ Error)

### 3. Borders and Lines (Not Material 3 Flat 2.0) ✅
**Problem:** Tool call blocks used borders and lines instead of background color differentiation.

**Fix:** Redesigned to use layered surface containers:
```typescript
// Before: borders everywhere
border border-panelBorder
border-b border-panelBorder

// After: background color zones
bg-surfaceContainer          // Main container
bg-surfaceContainerHigh      // Header
bg-surfaceContainerLow       // Arguments/Result sections
bg-surfaceContainerHighest   // Code blocks
```

**Design Principles:**
- No visible borders or lines
- Separation via background color hierarchy
- Surface container levels create visual depth
- Hover states use next level up in hierarchy

### 4. Copy Button Contrast (WCAG Non-Compliant) ✅
**Problem:** Copy button had insufficient contrast against backgrounds.

**Fix:**
```typescript
// Before
bg-panel/80 text-textMuted  // Low contrast

// After
bg-surfaceContainerHighest text-textPrimary shadow-sm  // High contrast
opacity-70 hover:opacity-100  // Always visible but subtle
```

**Improvements:**
- Solid background with shadow for depth
- High-contrast text color
- 70% opacity base (visible but not distracting)
- 100% opacity on hover
- Hover changes background to `bg-primary/20` with `text-primary`

### 5. Arguments and Results in Same Space ✅
**Problem:** Tool call results were not shown in the same component as arguments.

**Fix:** Combined display with two distinct sections:
```typescript
<div class="bg-surfaceContainerLow rounded-lg p-3">
  <div class="text-xs font-medium text-textSecondary mb-2">Arguments</div>
  <pre class="bg-surfaceContainerHighest ...">
    <code>${formattedArgs}</code>
  </pre>
  <copy-button ...></copy-button>
</div>

${resultData ? `
  <div class="bg-surfaceContainerLow rounded-lg p-3">
    <div class="text-xs font-medium text-textSecondary mb-2">Result</div>
    <pre class="bg-surfaceContainerHighest ...">
      <code>${resultData.content}</code>
    </pre>
    <copy-button ...></copy-button>
  </div>
` : ''}
```

**Features:**
- Both sections use same background color (`surfaceContainerLow`)
- Clear labels: "Arguments" and "Result"
- Each has its own copy button
- Result only shows when available
- Consistent spacing and styling

### 6. Tool Calls Centered ✅
**Problem:** Tool calls appeared left-aligned and lopsided.

**Fix:**
```typescript
return `
  <div class="flex justify-center my-4">
    <chat-tool-call ...></chat-tool-call>
  </div>
`;
```

## Files Modified

### Frontend Components
1. **`web/components/chat-stream/chat-stream.ts`**
   - Fixed `renderToolCall()` attribute mapping
   - Added `ToolCallItem` type import
   - Added `createUniqueId` import
   - Updated copy button opacity (0 → 70) for thinking/reasoning blocks

2. **`web/components/chat-tool-call/chat-tool-call.ts`**
   - Complete redesign to Material 3 Flat 2.0
   - Removed all borders
   - Implemented surface container hierarchy
   - Combined arguments and results in same view
   - Fixed status badge logic

3. **`web/components/copy-button/copy-button.ts`**
   - Updated background: `bg-surfaceContainerHighest`
   - Updated text color: `text-textPrimary`
   - Added shadow for depth
   - Improved hover state with primary color

## Visual Hierarchy

### Surface Container Levels (Dark → Light in Dark Mode)
```
surfaceContainerHighest  ← Code blocks (lightest)
    ↑
surfaceContainerHigh     ← Tool call header
    ↑
surfaceContainer         ← Tool call main body
    ↑
surfaceContainerLow      ← Arguments/Result sections
    ↑
surface                  ← Page background (darkest)
```

### Color Differentiation
- **No borders** - All separation via background colors
- **Sufficient contrast** - WCAG AA compliant
- **Hover feedback** - Next level up in hierarchy
- **Status badges** - Color-coded (success/warning/danger)

## Accordion Behavior

Tool calls are collapsible:
- **Collapsed:** Shows header with tool name and status badge
- **Expanded:** Shows full arguments and results
- **Chevron icon:** Rotates to indicate state
- **Click header:** Toggles expansion
- **Smooth animation:** 300ms ease-in-out transition

## Status Indicators

1. **Streaming:** Yellow badge with pulse animation
2. **Complete:** Green badge (no result yet)
3. **Success:** Green badge with ✓ checkmark (result received)
4. **Error:** Red badge with ✗ mark (result received)

## Testing Checklist

- [x] Tool call arguments display correctly
- [x] Status updates from "Streaming..." to "Complete"
- [x] Result appears in same component when available
- [x] No borders or lines visible
- [x] Copy buttons have sufficient contrast
- [x] Copy buttons work for both arguments and results
- [x] Tool calls are centered in chat flow
- [x] Accordion expand/collapse works
- [x] All chunk blocks (thinking, reasoning, tool calls) use consistent design
- [x] Light and dark modes both WCAG compliant

## Related Documentation

- `docs/UI_DESIGN.md` - Material 3 Flat 2.0 design system
- `docs/STREAMING_VERIFICATION.md` - Real-time event handling
- `docs/STATE_MANAGEMENT.md` - PGlite persistence
