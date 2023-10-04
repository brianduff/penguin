import { useLoaderData, useRevalidator } from "react-router-dom";
import { Result } from "./result";
import { DomainList } from "./bindings/DomainList";
import { Delete, GlobeNetwork } from "@blueprintjs/icons";
import { Button, Dialog, DialogBody, DialogFooter, Section, SectionCard, TextArea } from "@blueprintjs/core";
import { Table } from "./components/Table";
import { ChangeEvent, useState } from "react";
import { DNSInputField } from "./Domains";
import { FieldEditor, clone } from "./components/FieldEditor";
import { updateDomainList } from "./api";
import { css } from "@emotion/react";

export function ViewDomains() {
  const domains = useLoaderData() as Result<DomainList>;
  const [errorMessage, setErrorMessage] = useState("");
  const [newDomain, setNewDomain] = useState("");
  const revalidator = useRevalidator();
  const [isBulkOpen, setBulkOpen] = useState(false);
  const [bulkDomains, setBulkDomains] = useState("");

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

  const commitList = async (dl: Object) => {
    return await (await updateDomainList(dl as DomainList)).andThen(revalidate);
  }

  const submitBulkDomains = async () => {
    const newDomain = clone(domains.unwrap());
    if (newDomain.domains === undefined) {
      newDomain.domains = [];
    }

    const closeDialog = async (dl: DomainList) : Promise<Result<DomainList>> => {
      setBulkOpen(false);
      return Result.Ok(dl);
    }

    newDomain.domains.push(...bulkDomains.split("\n").filter(s => s !== null && s.length !== 0).map(s => s.trim()));
    return (await (await updateDomainList(newDomain)).andThen(revalidate)).andThen(closeDialog);
  }

  return (
    <Section title="Entries" icon={<GlobeNetwork />}>
      <SectionCard>
        <div css={css`display: grid; grid-template-columns: auto 1fr; grid-gap: 10px;`}>
          <span>Name:</span>
          <FieldEditor onSubmit={commitList} field="name" original={domains.unwrap()} />
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
        &nbsp;
        <Button onClick={() => setBulkOpen(true)}>Bulk Add...</Button>
        <Dialog
              isOpen={isBulkOpen}
              title="Bulk add domains"
              onClose={() => setBulkOpen(false)}>
            <DialogBody>
              <p>Enter domains to add one line at a time.</p>
              <p>
                <TextArea
                    value={bulkDomains}
                    onChange={(e: ChangeEvent<HTMLTextAreaElement>) => setBulkDomains(e.target.value)}
                    autoResize={true} />
              </p>
            </DialogBody>
            <DialogFooter>
              <Button intent="primary" text="Add" onClick={submitBulkDomains} />
            </DialogFooter>
        </Dialog>
      </SectionCard>
    </Section>);

}