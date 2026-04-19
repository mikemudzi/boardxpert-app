// src/screens/JobsListScreen.tsx
import React from 'react';
import { View, FlatList, StyleSheet } from 'react-native';
import { FAB, Text, Portal, Dialog, Button } from 'react-native-paper';
import { useNavigation } from '@react-navigation/native';
import { StackNavigationProp } from '@react-navigation/stack';
import { useStore } from '../store';
import { RootStackParamList } from '../navigation/AppNavigator';
import JobCard from '../components/JobCard';

type NavigationProp = StackNavigationProp<RootStackParamList, 'JobsList'>;

export default function JobsListScreen() {
  const navigation = useNavigation<NavigationProp>();
  const { jobs, createJob, deleteJob, setCurrentJob } = useStore();
  const [deleteDialogVisible, setDeleteDialogVisible] = React.useState(false);
  const [jobToDelete, setJobToDelete] = React.useState<string | null>(null);

  const handleCreateJob = () => {
    const jobId = createJob();
    setCurrentJob(jobId);
    navigation.navigate('JobEditor', { jobId });
  };

  const handleOpenJob = (jobId: string) => {
    setCurrentJob(jobId);
    navigation.navigate('JobEditor', { jobId });
  };

  const handleDeletePress = (jobId: string) => {
    setJobToDelete(jobId);
    setDeleteDialogVisible(true);
  };

  const handleConfirmDelete = () => {
    if (jobToDelete) {
      deleteJob(jobToDelete);
    }
    setDeleteDialogVisible(false);
    setJobToDelete(null);
  };

  return (
    <View style={styles.container}>
      {jobs.length === 0 ? (
        <View style={styles.empty}>
          <Text variant="titleMedium">No jobs yet</Text>
          <Text variant="bodyMedium" style={styles.emptySubtext}>
            Tap the + button to create your first job
          </Text>
        </View>
      ) : (
        <FlatList
          data={jobs}
          keyExtractor={(item) => item.id}
          renderItem={({ item }) => (
            <JobCard
              job={item}
              onPress={() => handleOpenJob(item.id)}
              onDelete={() => handleDeletePress(item.id)}
            />
          )}
          contentContainerStyle={styles.list}
        />
      )}

      <FAB
        icon="plus"
        style={styles.fab}
        onPress={handleCreateJob}
      />

      <Portal>
        <Dialog visible={deleteDialogVisible} onDismiss={() => setDeleteDialogVisible(false)}>
          <Dialog.Title>Delete Job</Dialog.Title>
          <Dialog.Content>
            <Text>Are you sure you want to delete this job?</Text>
          </Dialog.Content>
          <Dialog.Actions>
            <Button onPress={() => setDeleteDialogVisible(false)}>Cancel</Button>
            <Button onPress={handleConfirmDelete}>Delete</Button>
          </Dialog.Actions>
        </Dialog>
      </Portal>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
  },
  list: {
    paddingVertical: 8,
  },
  empty: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
  },
  emptySubtext: {
    color: '#666',
    marginTop: 8,
  },
  fab: {
    position: 'absolute',
    right: 16,
    bottom: 16,
  },
});
