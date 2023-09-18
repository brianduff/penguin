import { HTMLTable, Section, SectionCard } from "@blueprintjs/core";
import { GlobeNetwork } from "@blueprintjs/icons";

export function Domains() {
  return (
    <Section title="Domain Lists" icon={<GlobeNetwork />}>
      <SectionCard>
        <HTMLTable compact={true} striped={true}>
          <thead>
            <tr>
              <th>Name</th>
              <th>Domain count</th>
            </tr>
          </thead>
        </HTMLTable>
      </SectionCard>
    </Section>
  )
}