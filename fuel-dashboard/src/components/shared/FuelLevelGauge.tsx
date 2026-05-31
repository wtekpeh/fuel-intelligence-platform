import ReactSpeedometer from "react-d3-speedometer";

interface FuelLevelGaugeProps {
  value: number;
  maxValue?: number;
  size?: "compact" | "large";
  label?: string;
}

function FuelLevelGauge({
  value,
  maxValue = 100,
  size = "large",
  label = "Fuel Level",
}: FuelLevelGaugeProps) {
  const normalizedValue = Math.max(0, Math.min(value, maxValue));

  const width = size === "compact" ? 180 : 280;
  const height = size === "compact" ? 120 : 180;

  return (
    <div className={`fuel-level-gauge fuel-level-gauge--${size}`}>
      <p>{label}</p>

      <ReactSpeedometer
        value={normalizedValue}
        minValue={0}
        maxValue={maxValue}
        width={width}
        height={height}
        needleHeightRatio={0.75}
        currentValueText={`${normalizedValue.toFixed(1)}L`}
        customSegmentLabels={[]}
        ringWidth={size === "compact" ? 18 : 26}
      />
    </div>
  );
}

export default FuelLevelGauge;
