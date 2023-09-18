import { Section, SectionCard } from "@blueprintjs/core";
import { GlobeNetwork } from "@blueprintjs/icons";
import { Table } from "./components/Table";

export function Domains() {
  return (
    <Section title="Domain Lists" icon={<GlobeNetwork />}>
      <SectionCard>
        <Table columnNames={["Name", "Domain count"]}>

        </Table>
      </SectionCard>
    </Section>
  )
}