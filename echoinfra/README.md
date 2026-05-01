# EchoInfra - Infrastructure Monitoring Product

## Structure
```
echoinfra/
├── frontend/       - TypeScript/React
├── scada/         - Python SCADA bridge
├── degradation/   - Python degradation models
└── retrofit/      - Python retrofit planner
```

## SCADA Tiers
- **Tier 1**: Ruggedized x86/ARM (Siemens SIMATIC, Beckhoff CX)
- **Tier 2**: Jetson AGX (optional, local inference)

## Degradation Models
- Bridge: corrosion SDE + jump process (seismic)
- Pipeline: porous media flow + thermal SDE
- Grid: thermo-EM PDE + geomagnetic SDE
- Dam: seepage PDE + precip SDE

## Visualization
- DegradationTrajectory component
- Separate aleatoric vs epistemic envelopes
- Regime flags: 'calibrated' | 'ood' | 'regime_shift'