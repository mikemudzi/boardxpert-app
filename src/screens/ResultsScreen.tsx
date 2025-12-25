// src/screens/ResultsScreen.tsx
import React, { useState } from 'react';
import { View, ScrollView, StyleSheet } from 'react-native';
import { Text, Button, Portal, Modal, List } from 'react-native-paper';
import { useRoute, useNavigation, RouteProp } from '@react-navigation/native';
import { StackNavigationProp } from '@react-navigation/stack';
import { useStore } from '../store';
import { RootStackParamList } from '../navigation/AppNavigator';
import LayoutDiagram from '../components/LayoutDiagram';
import SummaryStats from '../components/SummaryStats';
import { PlacedPiece } from '../types';
import { resetColorCache } from '../utils/colors';

type RouteProps = RouteProp<RootStackParamList, 'Results'>;
type NavigationProp = StackNavigationProp<RootStackParamList, 'Results'>;

export default function ResultsScreen() {
  const route = useRoute<RouteProps>();
  const navigation = useNavigation<NavigationProp>();
  const { jobId } = route.params;

  const job = useStore((state) => state.jobs.find((j) => j.id === jobId));

  const [currentSheetIndex, setCurrentSheetIndex] = useState(0);
  const [selectedPiece, setSelectedPiece] = useState<PlacedPiece | null>(null);

  React.useEffect(() => {
    // Reset color cache when viewing new results
    resetColorCache();
  }, [jobId]);

  if (!job || !job.result) {
    return (
      <View style={styles.container}>
        <Text>No results available</Text>
        <Button onPress={() => navigation.goBack()}>Go Back</Button>
      </View>
    );
  }

  const { result } = job;
  const currentLayout = result.layouts[currentSheetIndex];
  const stockSheet = job.stockSheets[0]; // For now, use first stock sheet

  const handlePiecePress = (pieceId: string) => {
    const piece = currentLayout.pieces.find((p) => p.piece_id === pieceId);
    setSelectedPiece(piece || null);
  };

  const handleDone = () => {
    navigation.navigate('JobsList');
  };

  return (
    <View style={styles.container}>
      <ScrollView>
        <SummaryStats result={result} />

        {result.layouts.length > 1 && (
          <View style={styles.sheetSelector}>
            <Button
              disabled={currentSheetIndex === 0}
              onPress={() => setCurrentSheetIndex((i) => i - 1)}
            >
              Previous
            </Button>
            <Text variant="titleMedium">
              Sheet {currentSheetIndex + 1} of {result.layouts.length}
            </Text>
            <Button
              disabled={currentSheetIndex === result.layouts.length - 1}
              onPress={() => setCurrentSheetIndex((i) => i + 1)}
            >
              Next
            </Button>
          </View>
        )}

        <LayoutDiagram
          layout={currentLayout}
          sheetWidth={stockSheet?.width || 2440}
          sheetLength={stockSheet?.length || 1220}
          selectedPieceId={selectedPiece?.piece_id}
          onPiecePress={handlePiecePress}
        />

        <Text variant="bodySmall" style={styles.wasteText}>
          Waste: {currentLayout.waste_percentage.toFixed(1)}%
        </Text>

        <Text variant="titleSmall" style={styles.listTitle}>
          Pieces on this sheet ({currentLayout.pieces.length})
        </Text>
        {currentLayout.pieces.map((piece) => (
          <List.Item
            key={piece.piece_id}
            title={piece.piece_id}
            description={`${piece.width} x ${piece.length} at (${piece.x}, ${piece.y})${piece.rotated ? ' - Rotated' : ''}`}
            onPress={() => handlePiecePress(piece.piece_id)}
            style={piece.piece_id === selectedPiece?.piece_id ? styles.selectedItem : undefined}
          />
        ))}
      </ScrollView>

      <View style={styles.footer}>
        <Button mode="contained" onPress={handleDone}>
          Done
        </Button>
      </View>

      <Portal>
        <Modal
          visible={!!selectedPiece}
          onDismiss={() => setSelectedPiece(null)}
          contentContainerStyle={styles.modal}
        >
          {selectedPiece && (
            <>
              <Text variant="titleMedium">{selectedPiece.piece_id}</Text>
              <Text>Dimensions: {selectedPiece.width} x {selectedPiece.length}</Text>
              <Text>Position: ({selectedPiece.x}, {selectedPiece.y})</Text>
              <Text>Rotated: {selectedPiece.rotated ? 'Yes' : 'No'}</Text>
              {selectedPiece.label && <Text>Label: {selectedPiece.label}</Text>}
              <Button onPress={() => setSelectedPiece(null)} style={styles.closeButton}>
                Close
              </Button>
            </>
          )}
        </Modal>
      </Portal>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
  },
  sheetSelector: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    paddingHorizontal: 16,
    marginVertical: 8,
  },
  wasteText: {
    textAlign: 'center',
    color: '#666',
    marginBottom: 16,
  },
  listTitle: {
    paddingHorizontal: 16,
    marginTop: 16,
    marginBottom: 8,
    fontWeight: 'bold',
  },
  selectedItem: {
    backgroundColor: '#E3F2FD',
  },
  footer: {
    padding: 16,
    borderTopWidth: 1,
    borderTopColor: '#E0E0E0',
  },
  modal: {
    backgroundColor: 'white',
    padding: 20,
    margin: 20,
    borderRadius: 8,
  },
  closeButton: {
    marginTop: 16,
  },
});
