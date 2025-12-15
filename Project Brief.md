# 2D Cut Optimization System

A complete solution for optimizing 2D cutting layouts for sheet materials, designed for carpenters, woodworkers, and manufacturing environments.

## Overview

This project consists of two main components:

1. **Cut Optimizer API** (Rust) - A high-performance REST API for 2D cutting stock optimization
2. **Cut Optimizer Mobile** (React Native/Expo) - A mobile application for carpenters to use in the field

The optimization algorithm is based on the academic paper:
> "An Algorithm for the Two-Dimensional Cutting-Stock Problem Based on a Pattern Generation Procedure"
> by Ahmed Mellouli and Abdelaziz Dammak (2008)

## Features

### API Features
- 2D bin packing optimization using heuristic algorithms
- Guillotine cutting constraint support
- Multiple stock sheet sizes and materials
- Blade kerf/saw width consideration
- Edge banding tracking
- PDF layout generation
- SVG visualization output
- High-performance Rust implementation
- RESTful API with JSON responses
- CORS support for web applications
- Request validation

### Mobile App Features
- Intuitive piece entry interface
- Stock sheet management
- Real-time optimization
- Visual layout previews
- Edge banding configuration
- Job saving and loading
- Share optimization results
- Offline cutting list access
- Cross-platform (iOS & Android)

## Architecture

```
┌─────────────────┐     HTTP/JSON      ┌─────────────────┐
│                 │◄─────────────────► │                 │
│  Mobile App     │                    │  Cut Optimizer  │
│  (React Native) │                    │  API (Rust)     │
│                 │                    │                 │
└─────────────────┘                    └─────────────────┘
                                              │
                                              ▼
                                       ┌─────────────────┐
                                       │                 │
                                       │  Optimization   │
                                       │  Engine         │
                                       │                 │
                                       └─────────────────┘
```

## API Documentation

### Endpoints

#### Health Check
```
GET /health
```
Returns server status, version, and uptime.

#### Get Stock Sheet Templates
```
GET /api/v1/templates
```
Returns predefined stock sheet templates (Melamine, MDF, Plywood, etc.)

#### Validate Request
```
POST /api/v1/validate
```
Validates an optimization request without running optimization.

#### Run Optimization
```
POST /api/v1/optimize
```
Runs the full optimization algorithm.

#### Quick Optimization
```
POST /api/v1/optimize/quick
```
Runs optimization with a reduced timeout (max 10 seconds).

### Request Format

```json
{
  "job_reference": "JOB-12345",
  "client_name": "John Smith",
  "cut_pieces": [
    {
      "id": "panel-a",
      "width": 580,
      "length": 2450,
      "quantity": 6,
      "label": "Side Panel A",
      "can_rotate": true,
      "edge_banding": {
        "top": true,
        "bottom": true,
        "left": false,
        "right": false,
        "material": "White 1mm"
      }
    }
  ],
  "stock_sheets": [
    {
      "id": "sheet-1",
      "name": "BOARD White",
      "width": 2740,
      "length": 1820,
      "thickness": 16,
      "quantity": null,
      "cost": 50.00
    }
  ],
  "parameters": {
    "blade_kerf": 4.0,
    "edge_margin": 0,
    "max_transversal_cuts": 6,
    "guillotine_cuts": true,
    "priority": "minimize_waste",
    "timeout_seconds": 30
  },
  "output": {
    "generate_pdf": true,
    "generate_svg": true,
    "include_cutting_list": true,
    "units": "millimeters",
    "paper_size": "a4"
  }
}
```

### Response Format

```json
{
  "success": true,
  "result": {
    "job_id": "uuid-...",
    "job_reference": "JOB-12345",
    "total_sheets": 5,
    "total_pieces": 27,
    "overall_waste_percentage": 28.5,
    "total_used_area": 15750000,
    "total_waste_area": 6250000,
    "layouts_by_material": [
      {
        "material_name": "BOARD White",
        "sheet_width": 2740,
        "sheet_length": 1820,
        "layouts": [
          {
            "layout_number": 1,
            "occurrences": 1,
            "pieces": [
              {
                "piece_id": "panel-a-1",
                "label": "Side Panel A",
                "x": 0,
                "y": 0,
                "width": 580,
                "length": 2450,
                "rotated": false
              }
            ],
            "waste_percentage": 25.3
          }
        ]
      }
    ],
    "summary": {
      "cutting_list": [...],
      "material_usage": [...]
    }
  }
}
```

## Algorithm Details

The optimization uses a three-step heuristic approach:

1. **Pattern Generation**: Enumerates all feasible non-dominated cutting patterns using a search tree
2. **Relaxation**: Removes integer constraints to obtain a linear programming formulation
3. **Solution Generation**: Converts the relaxed solution back to integer values

Key optimizations:
- First Fit Decreasing Height (FFDH) bin packing
- Guillotine split for free rectangle management
- Dominated pattern elimination for reduced search space

## Performance

The API should be designed to handle:
- 1000+ concurrent users
- Up to 100 piece types per optimization
- Up to 10,000 total pieces
- Response times under 30 seconds for typical jobs

Performance tips:
- Use fewer piece types with higher quantities (faster than many unique pieces)
- Enable rotation for pieces without grain direction
- Use the `/optimize/quick` endpoint for interactive previews

## Project Structure

```
cut-optimizer-api/
├── Cargo.toml
├── src/
│   ├── main.rs           # Entry point
│   ├── lib.rs            # Library exports
│   ├── optimizer/
│   │   ├── mod.rs
│   │   ├── types.rs      # Data structures
│   │   └── algorithm.rs  # Core algorithm
│   ├── api/
│   │   ├── mod.rs
│   │   ├── types.rs      # Request/Response types
│   │   └── handlers.rs   # Route handlers
│   └── pdf/
│       ├── mod.rs
│       └── generator.rs  # PDF generation

cut-optimizer-mobile/
├── package.json
├── App.tsx               # Entry point
├── src/
│   ├── types/            # TypeScript types
│   ├── services/         # API client
│   ├── store/            # Zustand state management
│   ├── components/       # Reusable UI components
│   └── screens/          # App screens
```

## Technology Choices

### Why Rust for the API?

1. **Performance**: The 2D cutting stock problem is NP-hard. Rust's zero-cost abstractions provide C-like performance.
2. **Memory Safety**: Eliminates common bugs without garbage collection pauses.
3. **Concurrency**: Async/await with Tokio handles thousands of concurrent requests efficiently.
4. **WebAssembly**: Future option to run optimization client-side.

### Why React Native + Expo for Mobile?

1. **Cross-platform**: Single codebase for iOS and Android.
2. **Rapid Development**: Hot reloading and Expo's managed workflow.
3. **Native Performance**: React Native bridges to native UI components.
4. **Familiar Stack**: JavaScript/TypeScript knowledge transfers from web development.

## References

- Mellouli, A., & Dammak, A. (2008). An Algorithm for the Two-Dimensional Cutting-Stock Problem Based on a Pattern Generation Procedure. *International Journal of Information and Management Sciences*, 19(2), 201-218.
- [cut-optimizer-2d](https://github.com/jasonrhansen/cut-optimizer-2d) - Rust library for 2D cut optimization
- [Gilmore-Gomory Algorithm](https://en.wikipedia.org/wiki/Cutting_stock_problem) - Classic approach to cutting stock problems
