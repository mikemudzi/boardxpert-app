// src/components/SummaryStats.tsx
import React from 'react';
import { View, StyleSheet } from 'react-native';
import { Text, Card } from 'react-native-paper';
import { OptimizeResult } from '../types';

interface SummaryStatsProps {
  result: OptimizeResult;
}

export default function SummaryStats({ result }: SummaryStatsProps) {
  return (
    <Card style={styles.card}>
      <Card.Content>
        <View style={styles.row}>
          <View style={styles.stat}>
            <Text variant="headlineMedium">{result.total_sheets}</Text>
            <Text variant="bodySmall">Sheets</Text>
          </View>
          <View style={styles.stat}>
            <Text variant="headlineMedium">{result.total_pieces}</Text>
            <Text variant="bodySmall">Pieces</Text>
          </View>
          <View style={styles.stat}>
            <Text variant="headlineMedium">{result.efficiency.toFixed(1)}%</Text>
            <Text variant="bodySmall">Efficiency</Text>
          </View>
        </View>
      </Card.Content>
    </Card>
  );
}

const styles = StyleSheet.create({
  card: {
    margin: 16,
  },
  row: {
    flexDirection: 'row',
    justifyContent: 'space-around',
  },
  stat: {
    alignItems: 'center',
  },
});
