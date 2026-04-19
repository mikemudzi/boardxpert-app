// src/components/PieceList.tsx
import React, { useState } from 'react';
import { View, StyleSheet } from 'react-native';
import { List, IconButton, TextInput, Portal, Modal, Button, Switch, Text } from 'react-native-paper';
import { CutPiece } from '../types';

interface PieceListProps {
  pieces: CutPiece[];
  onUpdate: (pieceId: string, updates: Partial<CutPiece>) => void;
  onDelete: (pieceId: string) => void;
}

export default function PieceList({ pieces, onUpdate, onDelete }: PieceListProps) {
  const [editingPiece, setEditingPiece] = useState<CutPiece | null>(null);
  const [editForm, setEditForm] = useState({
    width: '',
    length: '',
    quantity: '',
    label: '',
    canRotate: true
  });

  const handleEditPress = (piece: CutPiece) => {
    setEditingPiece(piece);
    setEditForm({
      width: piece.width.toString(),
      length: piece.length.toString(),
      quantity: piece.quantity.toString(),
      label: piece.label || '',
      canRotate: piece.canRotate,
    });
  };

  const handleSaveEdit = () => {
    if (editingPiece) {
      onUpdate(editingPiece.id, {
        width: parseInt(editForm.width, 10) || editingPiece.width,
        length: parseInt(editForm.length, 10) || editingPiece.length,
        quantity: parseInt(editForm.quantity, 10) || 1,
        label: editForm.label || undefined,
        canRotate: editForm.canRotate,
      });
      setEditingPiece(null);
    }
  };

  if (pieces.length === 0) {
    return (
      <View style={styles.empty}>
        <Text>No pieces added yet</Text>
      </View>
    );
  }

  return (
    <View>
      {pieces.map((piece) => (
        <List.Item
          key={piece.id}
          title={`${piece.width} x ${piece.length}`}
          description={`Qty: ${piece.quantity}${piece.label ? ` • ${piece.label}` : ''}`}
          left={(props) => <List.Icon {...props} icon="square-outline" />}
          right={(props) => (
            <View style={styles.actions}>
              <IconButton icon="pencil" onPress={() => handleEditPress(piece)} />
              <IconButton icon="delete" onPress={() => onDelete(piece.id)} />
            </View>
          )}
          style={styles.item}
        />
      ))}

      <Portal>
        <Modal
          visible={!!editingPiece}
          onDismiss={() => setEditingPiece(null)}
          contentContainerStyle={styles.modal}
        >
          <Text variant="titleMedium" style={styles.modalTitle}>Edit Piece</Text>
          <View style={styles.row}>
            <TextInput
              label="Width"
              value={editForm.width}
              onChangeText={(t) => setEditForm({ ...editForm, width: t })}
              keyboardType="numeric"
              mode="outlined"
              style={styles.halfInput}
            />
            <TextInput
              label="Length"
              value={editForm.length}
              onChangeText={(t) => setEditForm({ ...editForm, length: t })}
              keyboardType="numeric"
              mode="outlined"
              style={styles.halfInput}
            />
          </View>
          <TextInput
            label="Quantity"
            value={editForm.quantity}
            onChangeText={(t) => setEditForm({ ...editForm, quantity: t })}
            keyboardType="numeric"
            mode="outlined"
            style={styles.input}
          />
          <TextInput
            label="Label (optional)"
            value={editForm.label}
            onChangeText={(t) => setEditForm({ ...editForm, label: t })}
            mode="outlined"
            style={styles.input}
          />
          <View style={styles.switchRow}>
            <Text>Can Rotate</Text>
            <Switch
              value={editForm.canRotate}
              onValueChange={(v) => setEditForm({ ...editForm, canRotate: v })}
            />
          </View>
          <View style={styles.modalActions}>
            <Button onPress={() => setEditingPiece(null)}>Cancel</Button>
            <Button mode="contained" onPress={handleSaveEdit}>Save</Button>
          </View>
        </Modal>
      </Portal>
    </View>
  );
}

const styles = StyleSheet.create({
  empty: {
    padding: 16,
    alignItems: 'center',
  },
  item: {
    backgroundColor: '#fff',
    marginBottom: 1,
  },
  actions: {
    flexDirection: 'row',
  },
  modal: {
    backgroundColor: 'white',
    padding: 20,
    margin: 20,
    borderRadius: 8,
  },
  modalTitle: {
    marginBottom: 16,
  },
  row: {
    flexDirection: 'row',
    gap: 8,
  },
  halfInput: {
    flex: 1,
    marginBottom: 12,
  },
  input: {
    marginBottom: 12,
  },
  switchRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 16,
  },
  modalActions: {
    flexDirection: 'row',
    justifyContent: 'flex-end',
    gap: 8,
  },
});
