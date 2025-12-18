# Cut Optimizer Mobile App Design

## Overview

**Cut Optimizer Mobile** is a React Native/Expo app for carpenters to optimize cutting layouts in the field. It connects to the Cut Optimizer API.

## Tech Stack

- **Framework:** Expo SDK 50+ (managed workflow)
- **Language:** TypeScript
- **Navigation:** React Navigation (stack + drawer)
- **State:** Zustand with persist middleware
- **Storage:** AsyncStorage (via Zustand persist)
- **UI:** React Native Paper (Material Design)
- **Graphics:** react-native-svg for layout diagrams
- **HTTP:** fetch for API calls

## MVP Scope

**In scope:**
- Create/edit jobs with pieces and stock sheets
- Run optimization via API
- View interactive layout results
- Save/load jobs locally
- Basic settings (API URL, units)

**Out of scope for MVP:**
- Edge banding configuration
- PDF export/sharing
- Offline optimization
- User accounts/cloud sync

## Navigation Structure

```
DrawerNavigator
├── MainStack (default)
│   ├── JobsList (home screen)
│   ├── JobEditor
│   │   ├── JobDetails (reference, client name)
│   │   ├── PiecesTab (add/edit pieces)
│   │   └── StockTab (select stock sheets)
│   ├── Optimizing (loading screen)
│   └── Results
│       ├── LayoutView (SVG diagram)
│       └── CuttingList (table view)
└── Drawer Menu
    ├── Settings
    └── Templates (stock sheet presets)
```

### Screen Flow

1. **JobsList** - Cards showing saved jobs with reference, date, piece count. FAB "+" to create new.
2. **JobEditor** - Tab view for entering job details, pieces, and stock. "Optimize" button in header.
3. **Optimizing** - Loading spinner while API processes. Cancel button.
4. **Results** - Swipeable sheets showing each layout. Summary stats at top. "Save" to persist results with job.

### Key Interactions

- Long-press job card for delete/duplicate options
- Pull-to-refresh on jobs list
- Swipe between layout sheets in results
- Tap piece in SVG to highlight and show dimensions

## Data Models

### Zustand Store Structure

```typescript
interface AppStore {
  // Jobs slice
  jobs: Job[];
  currentJob: Job | null;
  createJob: () => void;
  updateJob: (id: string, updates: Partial<Job>) => void;
  deleteJob: (id: string) => void;
  loadJob: (id: string) => void;

  // Settings slice
  settings: Settings;
  updateSettings: (updates: Partial<Settings>) => void;
}
```

### Core Data Types

```typescript
interface Job {
  id: string;
  jobReference: string;
  clientName?: string;
  pieces: CutPiece[];
  stockSheets: StockSheet[];
  result?: OptimizeResult;
  createdAt: string;
  updatedAt: string;
}

interface CutPiece {
  id: string;
  width: number;
  length: number;
  quantity: number;
  label?: string;
  canRotate: boolean;
}

interface StockSheet {
  id: string;
  name: string;
  width: number;
  length: number;
  thickness?: number;
}

interface Settings {
  apiUrl: string;
  units: 'mm' | 'inches';
  defaultBladeKerf: number;
}
```

### Persistence

Zustand's `persist` middleware with AsyncStorage. Jobs and settings auto-save on change. Results are stored with their job.

## API Integration

### API Service Layer

```typescript
// services/api.ts
const api = {
  validate: (request: OptimizeRequest) => Promise<ValidationResult>,
  optimizeQuick: (request: OptimizeRequest) => Promise<OptimizeResult>,
  getTemplates: () => Promise<StockSheet[]>,
  health: () => Promise<HealthStatus>,
}
```

### Request Building

Transform app state to API format before calling. The API expects `pieces` (not `cutPieces`) and specific field names. A `buildRequest(job: Job, settings: Settings)` function handles this mapping.

### Error Handling

- Network errors: "Cannot connect to server" with retry button
- Validation errors: Show inline on the field that failed
- Timeout: "Optimization taking too long" with option to wait or cancel
- Server errors: Generic error with "Try again"

### Loading States

- `idle` - Ready to optimize
- `validating` - Quick check before full optimization
- `optimizing` - API processing (show progress)
- `error` - Display error message
- `success` - Show results

## Layout Visualization

### SVG Layout Renderer

```typescript
interface LayoutDiagramProps {
  layout: SheetLayout;
  sheetWidth: number;
  sheetLength: number;
  selectedPieceId?: string;
  onPiecePress?: (pieceId: string) => void;
}
```

### Rendering Approach

- Calculate scale factor to fit sheet in viewport (maintaining aspect ratio)
- Draw sheet background as rectangle (light gray)
- Draw each piece as filled rectangle (colored by piece type)
- Draw piece ID/label centered in each rectangle
- Selected piece gets highlighted border
- Pinch-to-zoom and pan for detail viewing

### Color Coding

- Different pieces get different colors from a palette
- Same piece type (same ID prefix) gets same color across sheets
- Waste areas shown in light red/pink

### Piece Info Modal

When a piece is tapped, show a bottom sheet with:
- Piece ID and label
- Dimensions (width x length)
- Position on sheet (x, y)
- Whether it was rotated

## Project Structure

```
cut-optimizer-mobile/
├── app.json
├── App.tsx
├── package.json
├── tsconfig.json
├── src/
│   ├── types/
│   │   └── index.ts
│   ├── store/
│   │   ├── index.ts
│   │   ├── jobsSlice.ts
│   │   └── settingsSlice.ts
│   ├── services/
│   │   └── api.ts
│   ├── navigation/
│   │   └── AppNavigator.tsx
│   ├── screens/
│   │   ├── JobsListScreen.tsx
│   │   ├── JobEditorScreen.tsx
│   │   ├── OptimizingScreen.tsx
│   │   ├── ResultsScreen.tsx
│   │   ├── SettingsScreen.tsx
│   │   └── TemplatesScreen.tsx
│   ├── components/
│   │   ├── PieceInput.tsx
│   │   ├── PieceList.tsx
│   │   ├── StockSheetPicker.tsx
│   │   ├── LayoutDiagram.tsx
│   │   ├── JobCard.tsx
│   │   └── SummaryStats.tsx
│   └── utils/
│       ├── buildRequest.ts
│       └── colors.ts
```

## UI/UX Design Decisions

- **Quick-add piece entry:** Minimal input (width x length), quantity defaults to 1. Tap to edit details.
- **Stack navigation with drawer:** Jobs list as home, "+" FAB for new job, drawer for settings.
- **Material Design:** React Native Paper for consistent, polished UI components.
- **Interactive layouts:** SVG-based for tap-to-select, zoom/pan capability.
