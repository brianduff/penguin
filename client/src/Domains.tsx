import { Section, SectionCard } from "@blueprintjs/core";
import { GlobeNetwork } from "@blueprintjs/icons";
import { Table } from "./components/Table";
import { getDomainLists } from "./api";
import { useQuery } from "react-query";
import { InputWithButton } from "./components/InputWithButton";
import { useState } from "react";

export function Domains() {
  const query = useQuery("domainlists", getDomainLists);
  const [newDomain, setNewDomain] = useState("");
  const [errorMessage, setErrorMessage] = useState("");

  const addDomain = async () => {};
  const updateNewDomain = (value: string) => {};

  return (
    <Section title="Sites" icon={<GlobeNetwork />}>
      <SectionCard>
        <Table columnNames={["Name", "Domains"]}>
          {query.data && query.data.isOk() && query.data.unwrap().map(list => (
            <tr key={list.id}>
              <td>{list.name}</td>
              <td><DomainsSummary domains={list.domains} /></td>
            </tr>
          ))}
        </Table>
      </SectionCard>
      <SectionCard>
        <InputWithButton
            prompt="New domain to block"
            value={newDomain}
            submit={addDomain}
            onValueUpdated={updateNewDomain}
            errorMessage={errorMessage}
            setErrorMessage={setErrorMessage}
        />
      </SectionCard>
    </Section>
  )
}

const MAX_SUMMARY_DOMAINS = 3;

interface DomainsSummaryProps {
  domains: Array<string>
}
function DomainsSummary({ domains }: DomainsSummaryProps) {
  let result = domains.slice(0, MAX_SUMMARY_DOMAINS).join(", ");

  let diff = domains.length - MAX_SUMMARY_DOMAINS;
  if (diff > 0) {
    result += `, and ${diff} more.`;
  }

  return <span>{result}</span>
}