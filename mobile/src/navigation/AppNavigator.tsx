// src/navigation/AppNavigator.tsx
import React from 'react';
import { NavigationContainer } from '@react-navigation/native';
import { createStackNavigator } from '@react-navigation/stack';
import { createDrawerNavigator } from '@react-navigation/drawer';

import JobsListScreen from '../screens/JobsListScreen';
import JobEditorScreen from '../screens/JobEditorScreen';
import OptimizingScreen from '../screens/OptimizingScreen';
import ResultsScreen from '../screens/ResultsScreen';
import SettingsScreen from '../screens/SettingsScreen';
import TemplatesScreen from '../screens/TemplatesScreen';

export type RootStackParamList = {
  JobsList: undefined;
  JobEditor: { jobId: string };
  Optimizing: { jobId: string };
  Results: { jobId: string };
};

export type DrawerParamList = {
  Main: undefined;
  Settings: undefined;
  Templates: undefined;
};

const Stack = createStackNavigator<RootStackParamList>();
const Drawer = createDrawerNavigator<DrawerParamList>();

function MainStack() {
  return (
    <Stack.Navigator>
      <Stack.Screen
        name="JobsList"
        component={JobsListScreen}
        options={{ title: 'My Jobs' }}
      />
      <Stack.Screen
        name="JobEditor"
        component={JobEditorScreen}
        options={{ title: 'Edit Job' }}
      />
      <Stack.Screen
        name="Optimizing"
        component={OptimizingScreen}
        options={{ title: 'Optimizing...', headerLeft: () => null }}
      />
      <Stack.Screen
        name="Results"
        component={ResultsScreen}
        options={{ title: 'Results' }}
      />
    </Stack.Navigator>
  );
}

export default function AppNavigator() {
  return (
    <NavigationContainer>
      <Drawer.Navigator>
        <Drawer.Screen
          name="Main"
          component={MainStack}
          options={{ headerShown: false, title: 'Jobs' }}
        />
        <Drawer.Screen
          name="Settings"
          component={SettingsScreen}
        />
        <Drawer.Screen
          name="Templates"
          component={TemplatesScreen}
        />
      </Drawer.Navigator>
    </NavigationContainer>
  );
}
