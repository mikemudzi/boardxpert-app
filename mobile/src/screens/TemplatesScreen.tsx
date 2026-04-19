// src/screens/TemplatesScreen.tsx
import React, { useState, useEffect } from 'react';
import { View, FlatList, StyleSheet } from 'react-native';
import { Card, Text, ActivityIndicator, Button } from 'react-native-paper';
import { useStore } from '../store';
import { api } from '../services/api';
import { StockSheet } from '../types';

export default function TemplatesScreen() {
  const [templates, setTemplates] = useState<StockSheet[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

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
    } catch (e: any) {
      setError(e.message || 'Failed to load templates');
    } finally {
      setLoading(false);
    }
  };

  if (loading) {
    return (
      <View style={styles.center}>
        <ActivityIndicator size="large" />
        <Text style={styles.loadingText}>Loading templates...</Text>
      </View>
    );
  }

  if (error) {
    return (
      <View style={styles.center}>
        <Text style={styles.errorText}>{error}</Text>
        <Button mode="contained" onPress={loadTemplates}>
          Retry
        </Button>
      </View>
    );
  }

  return (
    <FlatList
      data={templates}
      keyExtractor={(item) => item.id}
      contentContainerStyle={styles.list}
      renderItem={({ item }) => (
        <Card style={styles.card}>
          <Card.Title title={item.name} />
          <Card.Content>
            <Text>Dimensions: {item.width} x {item.length} mm</Text>
            {item.thickness && <Text>Thickness: {item.thickness} mm</Text>}
            {item.cost && <Text>Cost: ${item.cost.toFixed(2)}</Text>}
          </Card.Content>
        </Card>
      )}
      ListEmptyComponent={
        <View style={styles.center}>
          <Text>No templates available</Text>
        </View>
      }
    />
  );
}

const styles = StyleSheet.create({
  center: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    padding: 16,
  },
  loadingText: {
    marginTop: 16,
    color: '#666',
  },
  errorText: {
    color: '#B00020',
    marginBottom: 16,
    textAlign: 'center',
  },
  list: {
    padding: 16,
  },
  card: {
    marginBottom: 12,
  },
});
