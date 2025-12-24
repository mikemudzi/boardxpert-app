// src/services/api.ts
import { ApiResponse, OptimizeRequest, OptimizeResult, StockSheet } from '../types';

class ApiService {
  private baseUrl: string = 'http://localhost:8080';

  setBaseUrl(url: string) {
    this.baseUrl = url.replace(/\/$/, ''); // Remove trailing slash
  }

  async health(): Promise<{ status: string; version: string }> {
    const response = await fetch(`${this.baseUrl}/health`);
    if (!response.ok) {
      throw new Error('Server not reachable');
    }
    return response.json();
  }

  async getTemplates(): Promise<StockSheet[]> {
    const response = await fetch(`${this.baseUrl}/api/v1/templates`);
    if (!response.ok) {
      throw new Error('Failed to fetch templates');
    }
    const data: ApiResponse<StockSheet[]> = await response.json();
    if (!data.success || !data.result) {
      throw new Error(data.error?.message || 'Failed to fetch templates');
    }
    return data.result;
  }

  async validate(request: OptimizeRequest): Promise<void> {
    const response = await fetch(`${this.baseUrl}/api/v1/validate`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(request),
    });
    const data: ApiResponse<void> = await response.json();
    if (!data.success) {
      const error = new Error(data.error?.message || 'Validation failed');
      (error as any).code = data.error?.code;
      (error as any).field = data.error?.field;
      throw error;
    }
  }

  async optimizeQuick(request: OptimizeRequest): Promise<OptimizeResult> {
    const response = await fetch(`${this.baseUrl}/api/v1/optimize/quick`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(request),
    });
    const data: ApiResponse<OptimizeResult & { job_reference: string }> = await response.json();
    if (!data.success || !data.result) {
      const error = new Error(data.error?.message || 'Optimization failed');
      (error as any).code = data.error?.code;
      throw error;
    }
    return {
      total_sheets: data.result.total_sheets,
      total_pieces: data.result.total_pieces,
      efficiency: data.result.efficiency,
      layouts: data.result.layouts,
    };
  }
}

export const api = new ApiService();
