// src/components/PieceInput.tsx
import React, { useState } from 'react';
import { View, StyleSheet } from 'react-native';
import { TextInput, Button } from 'react-native-paper';
import { CutPiece } from '../types';

interface PieceInputProps {
  onAdd: (piece: Omit<CutPiece, 'id'>) => void;
}

export default function PieceInput({ onAdd }: PieceInputProps) {
  const [width, setWidth] = useState('');
  const [length, setLength] = useState('');

  const handleAdd = () => {
    const w = parseInt(width, 10);
    const l = parseInt(length, 10);

    if (w > 0 && l > 0) {
      onAdd({
        width: w,
        length: l,
        quantity: 1,
        canRotate: true,
      });
      setWidth('');
      setLength('');
    }
  };

  const isValid = parseInt(width, 10) > 0 && parseInt(length, 10) > 0;

  return (
    <View style={styles.container}>
      <TextInput
        label="Width"
        value={width}
        onChangeText={setWidth}
        keyboardType="numeric"
        mode="outlined"
        style={styles.dimensionInput}
      />
      <TextInput
        label="Length"
        value={length}
        onChangeText={setLength}
        keyboardType="numeric"
        mode="outlined"
        style={styles.dimensionInput}
      />
      <Button
        mode="contained"
        onPress={handleAdd}
        disabled={!isValid}
        style={styles.addButton}
      >
        Add
      </Button>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 8,
    marginBottom: 16,
  },
  dimensionInput: {
    flex: 1,
  },
  addButton: {
    marginTop: 6,
  },
});
