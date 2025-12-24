// src/utils/buildRequest.ts
import { Job, Settings, OptimizeRequest } from '../types';

export function buildOptimizeRequest(job: Job, settings: Settings): OptimizeRequest {
  return {
    job_reference: job.jobReference,
    client_name: job.clientName,
    pieces: job.pieces.map((p) => ({
      id: p.id,
      width: p.width,
      length: p.length,
      quantity: p.quantity,
      label: p.label,
      can_rotate: p.canRotate,
    })),
    stock_sheets: job.stockSheets.map((s) => ({
      id: s.id,
      name: s.name,
      width: s.width,
      length: s.length,
      thickness: s.thickness,
    })),
    parameters: {
      blade_kerf: settings.defaultBladeKerf,
    },
    output: {
      generate_pdf: false,
    },
  };
}
