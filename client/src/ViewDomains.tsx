import { useLoaderData, useRevalidator } from "react-router-dom";
import { Result } from "./result";
import { DomainList } from "./bindings/DomainList";
import { Delete, GlobeNetwork } from "@blueprintjs/icons";
import { Button, Section, SectionCard } from "@blueprintjs/core";
import { Table } from "./components/Table";
import { useState } from "react";
import { DNSInputField } from "./Domains";
import { clone } from "./ViewClient";
import { updateDomainList } from "./api";

export function ViewDomains() {
  const domains = useLoaderData() as Result<DomainList>;
  const [errorMessage, setErrorMessage] = useState("");
  const [newDomain, setNewDomain] = useState("");
  const revalidator = useRevalidator();

  const revalidate = async (value: DomainList) => {
    setNewDomain("");
    revalidator.revalidate();
    return Result.Ok(value);
  }

  const submitDomain = async (domain: string) => {
    const newDomain = clone(domains.unwrap());
    newDomain.domains.push(domain);
    return await (await updateDomainList(newDomain)).andThen(revalidate);
  };

  const deleteDomain = async (index: number) => {
    const newDomain = clone(domains.unwrap());
    newDomain.domains.splice(index, 1);
    return await (await updateDomainList(newDomain)).andThen(revalidate);
  };

  return (
    <Section title="Entries" icon={<GlobeNetwork />}>
      <SectionCard>
        {domains.unwrap().domains &&
        <Table columnNames={["DNS name", ""]}>
          {domains.unwrap().domains.map((domain, index) => (
            <tr key={domain}>
              <td>{domain}</td>
              <td><Button onClick={() => deleteDomain(index)} icon={<Delete />}></Button></td>
            </tr>
          ))}
        </Table>}
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