// src/components/StockSheetPicker.tsx
import React, { useState, useEffect } from 'react';
import { View, StyleSheet } from 'react-native';
import { List, IconButton, Button, TextInput, Portal, Modal, Text, ActivityIndicator } from 'react-native-paper';
import { StockSheet } from '../types';
import { api } from '../services/api';
import { useStore } from '../store';

interface StockSheetPickerProps {
  selectedSheets: StockSheet[];
  onAdd: (sheet: StockSheet) => void;
  onRemove: (sheetId: string) => void;
}

export default function StockSheetPicker({ selectedSheets, onAdd, onRemove }: StockSheetPickerProps) {
  const [templates, setTemplates] = useState<StockSheet[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [showCustomModal, setShowCustomModal] = useState(false);
  const [customSheet, setCustomSheet] = useState({ name: '', width: '', length: '', thickness: '' });

  const settings = useStore((state) => state.settings);

  useEffect(() => {
    loadTemplates();
  }, []);

  const loadTemplates = async () => {
    setLoading(true);
    setError(null);
    try {
      api.setBaseUrl(settings.apiUrl);
      const result = await api.getTemplates();
      setTemplates(result);
    } catch (e) {
      setError('Could not load templates');
    } finally {
      setLoading(false);
    }
  };

  const handleAddCustom = () => {
    const sheet: StockSheet = {
      id: `custom-${Date.now()}`,
      name: customSheet.name || 'Custom Sheet',
      width: parseInt(customSheet.width, 10) || 2440,
      length: parseInt(customSheet.length, 10) || 1220,
      thickness: parseInt(customSheet.thickness, 10) || undefined,
    };
    onAdd(sheet);
    setShowCustomModal(false);
    setCustomSheet({ name: '', width: '', length: '', thickness: '' });
  };

  const isTemplateSelected = (templateId: string) =>
    selectedSheets.some((s) => s.id === templateId);

  return (
    <View>
      <Text variant="titleSmall" style={styles.sectionTitle}>Selected Sheets</Text>
      {selectedSheets.length === 0 ? (
        <Text style={styles.emptyText}>No sheets selected</Text>
      ) : (
        selectedSheets.map((sheet) => (
          <List.Item
            key={sheet.id}
            title={sheet.name}
            description={`${sheet.width} x ${sheet.length}`}
            right={() => <IconButton icon="close" onPress={() => onRemove(sheet.id)} />}
          />
        ))
      )}

      <Text variant="titleSmall" style={styles.sectionTitle}>Available Templates</Text>
      {loading ? (
        <ActivityIndicator style={styles.loader} />
      ) : error ? (
        <View style={styles.errorContainer}>
          <Text style={styles.errorText}>{error}</Text>
          <Button onPress={loadTemplates}>Retry</Button>
        </View>
      ) : (
        templates.map((template) => (
          <List.Item
            key={template.id}
            title={template.name}
            description={`${template.width} x ${template.length}`}
            right={() => (
              <Button
                mode={isTemplateSelected(template.id) ? 'outlined' : 'contained'}
                onPress={() => isTemplateSelected(template.id)
                  ? onRemove(template.id)
                  : onAdd(template)
                }
              >
                {isTemplateSelected(template.id) ? 'Remove' : 'Add'}
              </Button>
            )}
          />
        ))
      )}

      <Button
        mode="outlined"
        onPress={() => setShowCustomModal(true)}
        style={styles.customButton}
        icon="plus"
      >
        Add Custom Sheet
      </Button>

      <Portal>
        <Modal
          visible={showCustomModal}
          onDismiss={() => setShowCustomModal(false)}
          contentContainerStyle={styles.modal}
        >
          <Text variant="titleMedium" style={styles.modalTitle}>Custom Sheet</Text>
          <TextInput
            label="Name"
            value={customSheet.name}
            onChangeText={(t) => setCustomSheet({ ...customSheet, name: t })}
            mode="outlined"
            style={styles.input}
          />
          <View style={styles.row}>
            <TextInput
              label="Width"
              value={customSheet.width}
              onChangeText={(t) => setCustomSheet({ ...customSheet, width: t })}
              keyboardType="numeric"
              mode="outlined"
              style={styles.halfInput}
            />
            <TextInput
              label="Length"
              value={customSheet.length}
              onChangeText={(t) => setCustomSheet({ ...customSheet, length: t })}
              keyboardType="numeric"
              mode="outlined"
              style={styles.halfInput}
            />
          </View>
          <TextInput
            label="Thickness (optional)"
            value={customSheet.thickness}
            onChangeText={(t) => setCustomSheet({ ...customSheet, thickness: t })}
            keyboardType="numeric"
            mode="outlined"
            style={styles.input}
          />
          <View style={styles.modalActions}>
            <Button onPress={() => setShowCustomModal(false)}>Cancel</Button>
            <Button mode="contained" onPress={handleAddCustom}>Add</Button>
          </View>
        </Modal>
      </Portal>
    </View>
  );
}

const styles = StyleSheet.create({
  sectionTitle: {
    marginTop: 16,
    marginBottom: 8,
    fontWeight: 'bold',
  },
  emptyText: {
    color: '#666',
    fontStyle: 'italic',
    marginBottom: 8,
  },
  loader: {
    marginVertical: 16,
  },
  errorContainer: {
    alignItems: 'center',
    marginVertical: 16,
  },
  errorText: {
    color: '#B00020',
    marginBottom: 8,
  },
  customButton: {
    marginTop: 16,
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
  modalActions: {
    flexDirection: 'row',
    justifyContent: 'flex-end',
    gap: 8,
  },
});
