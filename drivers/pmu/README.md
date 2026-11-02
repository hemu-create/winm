# PMU / SMC Driver Notes

b00t provides RTC, RNG, reset, and shutdown over HVC. A Windows driver is
needed for battery, charger, thermal sensors, fans, and lid state.

## Safety

- Thermal limits must be enforced before stress testing.
- Fanless M3 Air requires conservative limits.
- Unsupported sleep should be disabled cleanly.

## Open questions

- SMC command interface on M3.
- Thermal sensor locations and safe thresholds.
- Fan control curve and minimum safe duty.
