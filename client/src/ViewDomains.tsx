import { useLoaderData } from "react-router-dom";
import { Result } from "./result";
import { DomainList } from "./bindings/DomainList";
import { Delete, GlobeNetwork } from "@blueprintjs/icons";
import { Button, Section, SectionCard } from "@blueprintjs/core";
import { Table } from "./components/Table";
import { useState } from "react";
import { DNSInputField } from "./Domains";

export function ViewDomains() {
  const domains = useLoaderData() as Result<DomainList>;
  const [errorMessage, setErrorMessage] = useState("");
  const [newDomain, setNewDomain] = useState("");

  const submitDomain = async (domain: string) => {
    return Result.Ok("");
  };

  return (
    <Section title="Entries" icon={<GlobeNetwork />}>
      <SectionCard>
        <Table columnNames={["DNS name", ""]}>
          {domains.unwrap().domains.map(domain => (
            <tr key={domain}>
              <td>{domain}</td>
              <td><Button icon={<Delete />}></Button></td>
            </tr>
          ))}
        </Table>
      </SectionCard>
      <SectionCard>
        <DNSInputField
          newDomain={newDomain}
          setNewDomain={setNewDomain}
          errorMessage={errorMessage}
          setErrorMessage={setErrorMessage}
          submitDomain={submitDomain}
        />
      </SectionCard>
    </Section>);

}