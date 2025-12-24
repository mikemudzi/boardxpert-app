// src/utils/colors.ts

// Distinct colors for pieces
const PIECE_COLORS = [
  '#4CAF50', // Green
  '#2196F3', // Blue
  '#FF9800', // Orange
  '#9C27B0', // Purple
  '#00BCD4', // Cyan
  '#FFEB3B', // Yellow
  '#E91E63', // Pink
  '#795548', // Brown
  '#607D8B', // Blue Grey
  '#8BC34A', // Light Green
];

// Cache to ensure same piece type gets same color
const colorCache = new Map<string, string>();

export function getPieceColor(pieceId: string): string {
  // Extract base ID (without suffix like "-1", "-2")
  const baseId = pieceId.replace(/-\d+$/, '');

  if (colorCache.has(baseId)) {
    return colorCache.get(baseId)!;
  }

  const index = colorCache.size % PIECE_COLORS.length;
  const color = PIECE_COLORS[index];
  colorCache.set(baseId, color);
  return color;
}

export function resetColorCache(): void {
  colorCache.clear();
}

export const SHEET_BACKGROUND = '#E0E0E0';
export const WASTE_COLOR = '#FFCDD2';
export const SELECTED_BORDER = '#1976D2';
