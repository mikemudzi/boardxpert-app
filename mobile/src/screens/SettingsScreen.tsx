// src/screens/SettingsScreen.tsx
import React, { useState } from 'react';
import { View, StyleSheet, ScrollView } from 'react-native';
import { TextInput, Button, Text, Snackbar, SegmentedButtons } from 'react-native-paper';
import { useStore } from '../store';
import { api } from '../services/api';

export default function SettingsScreen() {
  const settings = useStore((state) => state.settings);
  const updateSettings = useStore((state) => state.updateSettings);

  const [apiUrl, setApiUrl] = useState(settings.apiUrl);
  const [bladeKerf, setBladeKerf] = useState(settings.defaultBladeKerf.toString());
  const [testing, setTesting] = useState(false);
  const [testResult, setTestResult] = useState<{ success: boolean; message: string } | null>(null);

  const handleTestConnection = async () => {
    setTesting(true);
    setTestResult(null);
    try {
      api.setBaseUrl(apiUrl);
      const health = await api.health();
      setTestResult({
        success: true,
        message: `Connected! Server version: ${health.version}`
      });
    } catch (e: any) {
      setTestResult({
        success: false,
        message: e.message || 'Connection failed'
      });
    } finally {
      setTesting(false);
    }
  };

  const handleSave = () => {
    updateSettings({
      apiUrl,
      defaultBladeKerf: parseInt(bladeKerf, 10) || 3,
    });
    setTestResult({ success: true, message: 'Settings saved!' });
  };

  return (
    <ScrollView style={styles.container}>
      <Text variant="titleMedium" style={styles.sectionTitle}>
        API Configuration
      </Text>

      <TextInput
        label="API URL"
        value={apiUrl}
        onChangeText={setApiUrl}
        mode="outlined"
        style={styles.input}
        placeholder="http://localhost:8080"
      />

      <Button
        mode="outlined"
        onPress={handleTestConnection}
        loading={testing}
        style={styles.testButton}
      >
        Test Connection
      </Button>

      <Text variant="titleMedium" style={styles.sectionTitle}>
        Cutting Parameters
      </Text>

      <TextInput
        label="Default Blade Kerf (mm)"
        value={bladeKerf}
        onChangeText={setBladeKerf}
        mode="outlined"
        keyboardType="numeric"
        style={styles.input}
      />

      <Text variant="titleMedium" style={styles.sectionTitle}>
        Units
      </Text>

      <SegmentedButtons
        value={settings.units}
        onValueChange={(value) => updateSettings({ units: value as 'mm' | 'inches' })}
        buttons={[
          { value: 'mm', label: 'Millimeters' },
          { value: 'inches', label: 'Inches' },
        ]}
        style={styles.segmented}
      />

      <Button
        mode="contained"
        onPress={handleSave}
        style={styles.saveButton}
      >
        Save Settings
      </Button>

      <Snackbar
        visible={!!testResult}
        onDismiss={() => setTestResult(null)}
        duration={3000}
        style={testResult?.success ? styles.successSnackbar : styles.errorSnackbar}
      >
        {testResult?.message}
      </Snackbar>
    </ScrollView>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    padding: 16,
  },
  sectionTitle: {
    marginTop: 16,
    marginBottom: 8,
    fontWeight: 'bold',
  },
  input: {
    marginBottom: 12,
  },
  testButton: {
    marginBottom: 16,
  },
  segmented: {
    marginBottom: 16,
  },
  saveButton: {
    marginTop: 24,
    marginBottom: 32,
  },
  successSnackbar: {
    backgroundColor: '#4CAF50',
  },
  errorSnackbar: {
    backgroundColor: '#B00020',
  },
});
