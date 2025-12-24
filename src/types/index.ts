// src/types/index.ts

// Core domain types
export interface CutPiece {
  id: string;
  width: number;
  length: number;
  quantity: number;
  label?: string;
  canRotate: boolean;
}

export interface StockSheet {
  id: string;
  name: string;
  width: number;
  length: number;
  thickness?: number;
  cost?: number;
}

export interface Job {
  id: string;
  jobReference: string;
  clientName?: string;
  pieces: CutPiece[];
  stockSheets: StockSheet[];
  result?: OptimizeResult;
  createdAt: string;
  updatedAt: string;
}

// API response types
export interface PlacedPiece {
  piece_id: string;
  x: number;
  y: number;
  width: number;
  length: number;
  rotated: boolean;
  label?: string;
}

export interface SheetLayout {
  sheet_number: number;
  pieces: PlacedPiece[];
  waste_percentage: number;
}

export interface OptimizeResult {
  total_sheets: number;
  total_pieces: number;
  efficiency: number;
  layouts: SheetLayout[];
}

export interface ApiResponse<T> {
  success: boolean;
  result?: T;
  error?: {
    code: string;
    message: string;
    field?: string;
  };
}

// Settings
export interface Settings {
  apiUrl: string;
  units: 'mm' | 'inches';
  defaultBladeKerf: number;
}

// API request types
export interface OptimizeRequest {
  job_reference: string;
  client_name?: string;
  pieces: Array<{
    id: string;
    width: number;
    length: number;
    quantity: number;
    label?: string;
    can_rotate: boolean;
  }>;
  stock_sheets: Array<{
    id: string;
    name: string;
    width: number;
    length: number;
    thickness?: number;
  }>;
  parameters: {
    blade_kerf: number;
  };
  output: {
    generate_pdf: boolean;
  };
}
