# Studio - Architecture + BIM Product

## Structure
```
studio/
├── frontend/         - TypeScript/React (Three.js/WebGL)
├── design_agents/    - Python ML pipeline
├── bim/             - IFC4 pipeline
└── iot/             - BACnet/MQTT sensor bridge
```

## Features
- 3D model visualization
- Compliance checking
- Uncertainty visualization
- BIM export (IFC4 LOD 300-400)

## Dependencies
- TypeScript types generated from boundary contracts
- OCaml topological gate MUST pass before IFC write