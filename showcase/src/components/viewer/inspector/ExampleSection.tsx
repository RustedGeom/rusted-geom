import { EXAMPLE_SUMMARIES, getCategoryForExample, getLabelForExample } from "@/lib/examples";
import type { ExampleKey } from "@/lib/viewer-types";
import { ToolIcon } from "../toolbar/ToolIcon";

interface ExampleSectionProps {
  activeExample: ExampleKey;
  activeCurveName: string;
  activeDegreeLabel: string;
  onOpenExampleBrowser: () => void;
}

export function ExampleSection({
  activeExample,
  activeCurveName,
  activeDegreeLabel,
  onOpenExampleBrowser,
}: ExampleSectionProps) {
  const categoryLabel = getCategoryForExample(activeExample);
  const exampleLabel = getLabelForExample(activeExample);

  return (
    <section className="inspector-section" aria-label="Example selection">
      <h2>Example</h2>
      <button
        type="button"
        className="example-trigger"
        onClick={onOpenExampleBrowser}
        aria-label="Browse examples"
        title="Browse Examples (⌘K)"
      >
        <span className="example-trigger-label">{exampleLabel}</span>
        {categoryLabel && (
          <span className="example-trigger-badge">{categoryLabel}</span>
        )}
        <ToolIcon>
          <path d="M6 4l4 4-4 4" />
        </ToolIcon>
      </button>
      <div className="inspector-readout">
        <span>Name</span>
        <output>{activeCurveName}</output>
      </div>
      <div className="inspector-readout">
        <span>Type</span>
        <output>{activeDegreeLabel}</output>
      </div>
      <p className="inspector-note">{EXAMPLE_SUMMARIES[activeExample]}</p>
    </section>
  );
}
