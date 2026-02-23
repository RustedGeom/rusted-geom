import type { ViewerPerformance } from "@/lib/viewer-types";

interface PerformanceSectionProps {
  perfStats: ViewerPerformance;
}

export function PerformanceSection({ perfStats }: PerformanceSectionProps) {
  return (
    <section className="inspector-section" aria-label="Performance metrics">
      <h2>Performance</h2>
      <div className="inspector-readout">
        <span>Load</span>
        <output>{perfStats.loadMs.toFixed(2)} ms</output>
      </div>
      <div className="inspector-readout">
        <span>Intersection</span>
        <output>{perfStats.intersectionMs.toFixed(2)} ms</output>
      </div>
    </section>
  );
}
