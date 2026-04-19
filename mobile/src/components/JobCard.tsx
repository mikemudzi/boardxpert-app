// src/components/JobCard.tsx
import React from 'react';
import { StyleSheet } from 'react-native';
import { Card, Text, IconButton } from 'react-native-paper';
import { Job } from '../types';

interface JobCardProps {
  job: Job;
  onPress: () => void;
  onDelete: () => void;
}

export default function JobCard({ job, onPress, onDelete }: JobCardProps) {
  const pieceCount = job.pieces.reduce((sum, p) => sum + p.quantity, 0);
  const date = new Date(job.updatedAt).toLocaleDateString();

  return (
    <Card style={styles.card} onPress={onPress}>
      <Card.Title
        title={job.jobReference}
        subtitle={job.clientName || 'No client'}
        right={(props) => (
          <IconButton {...props} icon="delete" onPress={onDelete} />
        )}
      />
      <Card.Content>
        <Text variant="bodyMedium">
          {pieceCount} pieces • {job.stockSheets.length} stock sheets
        </Text>
        <Text variant="bodySmall" style={styles.date}>
          Updated {date}
        </Text>
        {job.result && (
          <Text variant="bodySmall" style={styles.result}>
            Result: {job.result.total_sheets} sheets, {job.result.efficiency.toFixed(1)}% efficiency
          </Text>
        )}
      </Card.Content>
    </Card>
  );
}

const styles = StyleSheet.create({
  card: {
    marginHorizontal: 16,
    marginVertical: 8,
  },
  date: {
    color: '#666',
    marginTop: 4,
  },
  result: {
    color: '#4CAF50',
    marginTop: 4,
  },
});
