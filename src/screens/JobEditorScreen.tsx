// src/screens/JobEditorScreen.tsx
import React, { useState, useLayoutEffect } from 'react';
import { View, StyleSheet, ScrollView } from 'react-native';
import { TextInput, Button, SegmentedButtons, Text } from 'react-native-paper';
import { useRoute, useNavigation, RouteProp } from '@react-navigation/native';
import { StackNavigationProp } from '@react-navigation/stack';
import { useStore } from '../store';
import { RootStackParamList } from '../navigation/AppNavigator';

type RouteProps = RouteProp<RootStackParamList, 'JobEditor'>;
type NavigationProp = StackNavigationProp<RootStackParamList, 'JobEditor'>;

export default function JobEditorScreen() {
  const route = useRoute<RouteProps>();
  const navigation = useNavigation<NavigationProp>();
  const { jobId } = route.params;

  const job = useStore((state) => state.jobs.find((j) => j.id === jobId));
  const updateJob = useStore((state) => state.updateJob);

  const [activeTab, setActiveTab] = useState('details');

  useLayoutEffect(() => {
    navigation.setOptions({
      headerRight: () => (
        <Button
          mode="contained"
          onPress={handleOptimize}
          disabled={!canOptimize}
          style={{ marginRight: 8 }}
        >
          Optimize
        </Button>
      ),
    });
  }, [job]);

  if (!job) {
    return (
      <View style={styles.container}>
        <Text>Job not found</Text>
      </View>
    );
  }

  const canOptimize = job.pieces.length > 0 && job.stockSheets.length > 0;

  const handleOptimize = () => {
    navigation.navigate('Optimizing', { jobId });
  };

  const renderDetailsTab = () => (
    <View style={styles.tabContent}>
      <TextInput
        label="Job Reference"
        value={job.jobReference}
        onChangeText={(text) => updateJob(jobId, { jobReference: text })}
        mode="outlined"
        style={styles.input}
      />
      <TextInput
        label="Client Name (optional)"
        value={job.clientName || ''}
        onChangeText={(text) => updateJob(jobId, { clientName: text || undefined })}
        mode="outlined"
        style={styles.input}
      />
    </View>
  );

  const renderPiecesTab = () => (
    <View style={styles.tabContent}>
      <Text>Pieces tab - {job.pieces.length} pieces</Text>
      {/* Will be implemented in next task */}
    </View>
  );

  const renderStockTab = () => (
    <View style={styles.tabContent}>
      <Text>Stock tab - {job.stockSheets.length} sheets</Text>
      {/* Will be implemented in later task */}
    </View>
  );

  return (
    <View style={styles.container}>
      <SegmentedButtons
        value={activeTab}
        onValueChange={setActiveTab}
        buttons={[
          { value: 'details', label: 'Details' },
          { value: 'pieces', label: `Pieces (${job.pieces.length})` },
          { value: 'stock', label: `Stock (${job.stockSheets.length})` },
        ]}
        style={styles.tabs}
      />
      <ScrollView style={styles.scrollView}>
        {activeTab === 'details' && renderDetailsTab()}
        {activeTab === 'pieces' && renderPiecesTab()}
        {activeTab === 'stock' && renderStockTab()}
      </ScrollView>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
  },
  tabs: {
    margin: 16,
  },
  scrollView: {
    flex: 1,
  },
  tabContent: {
    padding: 16,
  },
  input: {
    marginBottom: 16,
  },
});
