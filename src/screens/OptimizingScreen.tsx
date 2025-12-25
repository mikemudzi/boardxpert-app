// src/screens/OptimizingScreen.tsx
import React, { useEffect, useState } from 'react';
import { View, StyleSheet } from 'react-native';
import { ActivityIndicator, Text, Button } from 'react-native-paper';
import { useRoute, useNavigation, RouteProp } from '@react-navigation/native';
import { StackNavigationProp } from '@react-navigation/stack';
import { useStore } from '../store';
import { api } from '../services/api';
import { buildOptimizeRequest } from '../utils/buildRequest';
import { RootStackParamList } from '../navigation/AppNavigator';

type RouteProps = RouteProp<RootStackParamList, 'Optimizing'>;
type NavigationProp = StackNavigationProp<RootStackParamList, 'Optimizing'>;

export default function OptimizingScreen() {
  const route = useRoute<RouteProps>();
  const navigation = useNavigation<NavigationProp>();
  const { jobId } = route.params;

  const job = useStore((state) => state.jobs.find((j) => j.id === jobId));
  const settings = useStore((state) => state.settings);
  const setJobResult = useStore((state) => state.setJobResult);

  const [error, setError] = useState<string | null>(null);
  const [isOptimizing, setIsOptimizing] = useState(true);

  useEffect(() => {
    runOptimization();
  }, []);

  const runOptimization = async () => {
    if (!job) {
      setError('Job not found');
      setIsOptimizing(false);
      return;
    }

    setIsOptimizing(true);
    setError(null);

    try {
      api.setBaseUrl(settings.apiUrl);
      const request = buildOptimizeRequest(job, settings);
      const result = await api.optimizeQuick(request);
      setJobResult(jobId, result);
      navigation.replace('Results', { jobId });
    } catch (e: any) {
      setError(e.message || 'Optimization failed');
      setIsOptimizing(false);
    }
  };

  const handleCancel = () => {
    navigation.goBack();
  };

  const handleRetry = () => {
    runOptimization();
  };

  return (
    <View style={styles.container}>
      {isOptimizing ? (
        <>
          <ActivityIndicator size="large" style={styles.spinner} />
          <Text variant="titleMedium" style={styles.text}>
            Optimizing your layout...
          </Text>
          <Text variant="bodyMedium" style={styles.subtext}>
            This may take a few seconds
          </Text>
          <Button onPress={handleCancel} style={styles.button}>
            Cancel
          </Button>
        </>
      ) : error ? (
        <>
          <Text variant="titleMedium" style={styles.errorText}>
            Optimization Failed
          </Text>
          <Text variant="bodyMedium" style={styles.subtext}>
            {error}
          </Text>
          <View style={styles.buttonRow}>
            <Button onPress={handleCancel}>Go Back</Button>
            <Button mode="contained" onPress={handleRetry}>
              Retry
            </Button>
          </View>
        </>
      ) : null}
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    padding: 32,
  },
  spinner: {
    marginBottom: 24,
  },
  text: {
    marginBottom: 8,
  },
  subtext: {
    color: '#666',
    marginBottom: 24,
    textAlign: 'center',
  },
  errorText: {
    color: '#B00020',
    marginBottom: 8,
  },
  button: {
    marginTop: 16,
  },
  buttonRow: {
    flexDirection: 'row',
    gap: 16,
    marginTop: 24,
  },
});
