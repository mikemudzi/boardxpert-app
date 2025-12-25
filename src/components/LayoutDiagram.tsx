// src/components/LayoutDiagram.tsx
import React from 'react';
import { View, StyleSheet, Dimensions } from 'react-native';
import Svg, { Rect, Text as SvgText, G } from 'react-native-svg';
import { SheetLayout, PlacedPiece } from '../types';
import { getPieceColor, SHEET_BACKGROUND, SELECTED_BORDER } from '../utils/colors';

interface LayoutDiagramProps {
  layout: SheetLayout;
  sheetWidth: number;
  sheetLength: number;
  selectedPieceId?: string;
  onPiecePress?: (pieceId: string) => void;
}

export default function LayoutDiagram({
  layout,
  sheetWidth,
  sheetLength,
  selectedPieceId,
  onPiecePress,
}: LayoutDiagramProps) {
  const screenWidth = Dimensions.get('window').width - 32; // Padding
  const maxHeight = 400;

  // Calculate scale to fit in viewport
  const scaleX = screenWidth / sheetWidth;
  const scaleY = maxHeight / sheetLength;
  const scale = Math.min(scaleX, scaleY);

  const viewWidth = sheetWidth * scale;
  const viewHeight = sheetLength * scale;

  const renderPiece = (piece: PlacedPiece) => {
    const x = piece.x * scale;
    const y = piece.y * scale;
    const width = piece.width * scale;
    const height = piece.length * scale;
    const isSelected = piece.piece_id === selectedPieceId;
    const color = getPieceColor(piece.piece_id);

    // Determine label based on space available
    const label = width > 40 && height > 20
      ? piece.piece_id.replace(/-\d+$/, '')
      : '';

    return (
      <G key={piece.piece_id} onPress={() => onPiecePress?.(piece.piece_id)}>
        <Rect
          x={x}
          y={y}
          width={width}
          height={height}
          fill={color}
          stroke={isSelected ? SELECTED_BORDER : '#333'}
          strokeWidth={isSelected ? 3 : 1}
        />
        {label && (
          <SvgText
            x={x + width / 2}
            y={y + height / 2}
            fill="#fff"
            fontSize={12}
            fontWeight="bold"
            textAnchor="middle"
            alignmentBaseline="middle"
          >
            {label}
          </SvgText>
        )}
      </G>
    );
  };

  return (
    <View style={styles.container}>
      <Svg width={viewWidth} height={viewHeight}>
        {/* Sheet background */}
        <Rect
          x={0}
          y={0}
          width={viewWidth}
          height={viewHeight}
          fill={SHEET_BACKGROUND}
          stroke="#999"
          strokeWidth={2}
        />
        {/* Pieces */}
        {layout.pieces.map(renderPiece)}
      </Svg>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    alignItems: 'center',
    paddingVertical: 16,
  },
});
