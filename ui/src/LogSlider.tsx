type LogSliderProps = {
  label: string;
  value: number;
  onChange: (v: number) => void;
  min: number;
  max: number;
};

export function LogSlider({
  label,
  value,
  onChange,
  min,
  max,
}: LogSliderProps) {
  const logValue = Math.log10(value);
  const logMin = Math.log10(min);
  const logMax = Math.log10(max);
  return (
    <label className="tm-fps">
      {label}:
      <input
        type="range"
        min={logMin}
        max={logMax}
        step={0.01}
        value={logValue}
        onChange={(e) => onChange(Math.round(10 ** Number(e.target.value)))}
      />
      <span>{value}</span>
    </label>
  );
}
