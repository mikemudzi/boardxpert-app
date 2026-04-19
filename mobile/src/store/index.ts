// src/store/index.ts
import { create } from 'zustand';
import { persist, createJSONStorage } from 'zustand/middleware';
import AsyncStorage from '@react-native-async-storage/async-storage';
import { v4 as uuidv4 } from 'uuid';
import { Job, CutPiece, StockSheet, Settings, OptimizeResult } from '../types';

interface AppState {
  // Jobs
  jobs: Job[];
  currentJobId: string | null;

  // Settings
  settings: Settings;

  // Job actions
  createJob: () => string;
  updateJob: (id: string, updates: Partial<Job>) => void;
  deleteJob: (id: string) => void;
  setCurrentJob: (id: string | null) => void;
  getCurrentJob: () => Job | null;

  // Piece actions
  addPiece: (jobId: string, piece: Omit<CutPiece, 'id'>) => void;
  updatePiece: (jobId: string, pieceId: string, updates: Partial<CutPiece>) => void;
  deletePiece: (jobId: string, pieceId: string) => void;

  // Stock sheet actions
  addStockSheet: (jobId: string, sheet: StockSheet) => void;
  removeStockSheet: (jobId: string, sheetId: string) => void;

  // Result actions
  setJobResult: (jobId: string, result: OptimizeResult) => void;

  // Settings actions
  updateSettings: (updates: Partial<Settings>) => void;
}

const defaultSettings: Settings = {
  apiUrl: 'http://localhost:8080',
  units: 'mm',
  defaultBladeKerf: 3,
};

export const useStore = create<AppState>()(
  persist(
    (set, get) => ({
      jobs: [],
      currentJobId: null,
      settings: defaultSettings,

      createJob: () => {
        const id = uuidv4();
        const now = new Date().toISOString();
        const newJob: Job = {
          id,
          jobReference: `JOB-${Date.now().toString(36).toUpperCase()}`,
          pieces: [],
          stockSheets: [],
          createdAt: now,
          updatedAt: now,
        };
        set((state) => ({ jobs: [newJob, ...state.jobs] }));
        return id;
      },

      updateJob: (id, updates) => {
        set((state) => ({
          jobs: state.jobs.map((job) =>
            job.id === id
              ? { ...job, ...updates, updatedAt: new Date().toISOString() }
              : job
          ),
        }));
      },

      deleteJob: (id) => {
        set((state) => ({
          jobs: state.jobs.filter((job) => job.id !== id),
          currentJobId: state.currentJobId === id ? null : state.currentJobId,
        }));
      },

      setCurrentJob: (id) => {
        set({ currentJobId: id });
      },

      getCurrentJob: () => {
        const state = get();
        return state.jobs.find((job) => job.id === state.currentJobId) || null;
      },

      addPiece: (jobId, piece) => {
        const newPiece: CutPiece = { ...piece, id: uuidv4() };
        set((state) => ({
          jobs: state.jobs.map((job) =>
            job.id === jobId
              ? { ...job, pieces: [...job.pieces, newPiece], updatedAt: new Date().toISOString() }
              : job
          ),
        }));
      },

      updatePiece: (jobId, pieceId, updates) => {
        set((state) => ({
          jobs: state.jobs.map((job) =>
            job.id === jobId
              ? {
                  ...job,
                  pieces: job.pieces.map((p) =>
                    p.id === pieceId ? { ...p, ...updates } : p
                  ),
                  updatedAt: new Date().toISOString(),
                }
              : job
          ),
        }));
      },

      deletePiece: (jobId, pieceId) => {
        set((state) => ({
          jobs: state.jobs.map((job) =>
            job.id === jobId
              ? {
                  ...job,
                  pieces: job.pieces.filter((p) => p.id !== pieceId),
                  updatedAt: new Date().toISOString(),
                }
              : job
          ),
        }));
      },

      addStockSheet: (jobId, sheet) => {
        set((state) => ({
          jobs: state.jobs.map((job) =>
            job.id === jobId
              ? { ...job, stockSheets: [...job.stockSheets, sheet], updatedAt: new Date().toISOString() }
              : job
          ),
        }));
      },

      removeStockSheet: (jobId, sheetId) => {
        set((state) => ({
          jobs: state.jobs.map((job) =>
            job.id === jobId
              ? {
                  ...job,
                  stockSheets: job.stockSheets.filter((s) => s.id !== sheetId),
                  updatedAt: new Date().toISOString(),
                }
              : job
          ),
        }));
      },

      setJobResult: (jobId, result) => {
        set((state) => ({
          jobs: state.jobs.map((job) =>
            job.id === jobId
              ? { ...job, result, updatedAt: new Date().toISOString() }
              : job
          ),
        }));
      },

      updateSettings: (updates) => {
        set((state) => ({
          settings: { ...state.settings, ...updates },
        }));
      },
    }),
    {
      name: 'cut-optimizer-storage',
      storage: createJSONStorage(() => AsyncStorage),
    }
  )
);
