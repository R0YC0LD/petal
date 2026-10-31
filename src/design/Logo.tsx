// Petal logosu: üç katmanlı yaprak. Katmanlar ayrı grup olduğundan intro'da sırayla belirebilir.
// Tek renk: currentColor; ön yaprak vurgu rengini kullanır.
import { motion } from "motion/react";

interface Props {
  size?: number;
  animate?: boolean;
  className?: string;
}

const LEAF = "M32 6c9.5 8.2 14.5 17.6 14.5 27.5 0 9.4-6 16.5-14.5 22.5C23.5 50 17.5 42.900 17.5 33.500 17.500 23.600 22.500 14.200 32 6z";

export default function Logo({ size = 96, animate = false, className }: Props) {
  const layer = (rotate: number, i: number, opacity: number, color: string) => (
    <motion.g
      key={i}
      style={{ transformOrigin: "32px 56px", opacity, color }}
      initial={animate ? { rotate: 0, opacity: 0, scale: 0.7 } : false}
      animate={{ rotate, opacity, scale: 1 }}
      transition={{ delay: 0.12 + i * 0.12, type: "spring", stiffness: 180, damping: 22 }}
    >
      <path d={LEAF} fill="currentColor" />
    </motion.g>
  );
  return (
    <svg width={size} height={size} viewBox="0 0 64 64" className={className} role="img" aria-label="Petal">
      {layer(-34, 0, 0.35, "var(--text)")}
      {layer(34, 1, 0.55, "var(--text)")}
      {layer(0, 2, 1, "var(--accent)")}
    </svg>
  );
}
