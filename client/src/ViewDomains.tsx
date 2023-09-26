import { useLoaderData, useRevalidator } from "react-router-dom";
import { Result } from "./result";
import { DomainList } from "./bindings/DomainList";
import { Delete, GlobeNetwork } from "@blueprintjs/icons";
import { Button, Section, SectionCard } from "@blueprintjs/core";
import { Table } from "./components/Table";
import { useState } from "react";
import { DNSInputField } from "./Domains";
import { clone } from "./components/FieldEditor";
import { updateDomainList } from "./api";
import { css } from "@emotion/react";

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
    if (newDomain.domains === undefined) {
      newDomain.domains = [];
    }
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
        <div css={css`display: grid; grid-template-columns: auto 1fr; grid-gap: 10px;`}>
          <span>Name:</span>
          {/* <FieldEditor onSubmit={commitClient} field="name" original={client} /> */}
        </div>


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