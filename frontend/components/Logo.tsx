type Props = {
  size?: number;
  className?: string;
};

const TICKS = [
  { x: 10, y: 14, height: 36, fill: "#a8a8b2", rotate: "rotate(-8 13 32)" },
  { x: 21, y: 10, height: 44, fill: "#75757f", rotate: "rotate(-8 24 32)" },
  { x: 32, y: 8, height: 48, fill: "#3f3f46", rotate: "rotate(-8 35 32)" },
  { x: 45, y: 12, height: 40, fill: "#15803d", rotate: "rotate(-8 48 32)" },
];

export function Logo({ size = 18, className }: Props) {
  return (
    <svg width={size} height={size} viewBox="0 0 64 64" className={className}>
      {TICKS.map((t) => (
        <rect
          key={t.x}
          x={t.x}
          y={t.y}
          width={6}
          height={t.height}
          rx={1.5}
          fill={t.fill}
          transform={t.rotate}
        />
      ))}
    </svg>
  );
}
