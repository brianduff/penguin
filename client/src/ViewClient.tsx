import { useLoaderData, useNavigate, useRevalidator, useRouteLoaderData } from "react-router-dom";
import { ErrorMessage } from "./components/ErrorMessage";
import { Result } from "./result";
import { Client } from "./bindings/Client";
import { Button, EditableText, MenuItem, Section, SectionCard } from "@blueprintjs/core";
import { css } from "@emotion/react";
import { Delete, Edit, Remove } from "@blueprintjs/icons";
import { MultiSelect, ItemPredicate, ItemRenderer } from "@blueprintjs/select";
import { useRef, useState } from "react";
import { deleteClient, updateClient } from "./api";
import { FieldEditor } from "./components/FieldEditor";
import { DomainList } from "./bindings/DomainList";
import { AppGridLoaderData } from "./main";
import { DomainsSummary } from "./Domains";
import { Table } from "./components/Table";

export function ViewClient() {
  const client = useLoaderData() as Result<Client>;


  return client.match({
    ok: client => <div><Grid client={client} /></div>,
    err: msg => <ErrorMessage message={msg} />
  })
}

interface Props {
  client: Client
}

function Grid({ client }: Props) {
  const revalidator = useRevalidator();
  const revalidate = async (value: Client) => {
    revalidator.revalidate();
    return Result.Ok(value);
  }

  function ClientDetails() {
    const nameRef = useRef<EditableText>(null);
    const navigate = useNavigate();

    const onClickEdit = () => {
      nameRef.current?.toggleEditing();
    }

    const commitClient = async (newClient: Object) => {
      return (await updateClient(newClient as Client)).andThen(revalidate);
    };

    const navigateToClients = async (value: unknown) => {
      navigate("/");
      return Result.Ok(value);
    }

    const onClickDelete = async () => {
      return (await (await deleteClient(client)).andThen(revalidate)).andThen(navigateToClients);
    }

    return (
      <Section title="Details" rightElement={
        <>
          <Button onClick={onClickDelete} icon={<Delete />} />
          <Button onClick={onClickEdit} icon={<Edit />} />
        </>
        }>
        <SectionCard>
          <div css={css`display: grid; grid-template-columns: auto 1fr; grid-gap: 10px;`}>
            <span>Name:</span>
            <FieldEditor onSubmit={commitClient} field="name" original={client} />
            <span>IP&nbsp;Address:</span>
            <FieldEditor onSubmit={commitClient} field="ip" original={client} />
          </div>
        </SectionCard>
      </Section>
    );
  }

  interface DomainListChooserProps {
    domains: Array<DomainList>,
    selectedDomains: Set<DomainList>,
    setSelectedDomains: (selected: Set<DomainList>) => void,
    add: () => Promise<void>,
    remove: (dlid: number) => Promise<void>
  }

  function DomainListChooser({ domains, selectedDomains, setSelectedDomains, add, remove }: DomainListChooserProps) {

    const filterDomainLists: ItemPredicate<DomainList> = (query, dl, _index, exactMatch) => {
      const normTitle = dl.name.toLowerCase();
      const normQuery = query.toLowerCase();

      if (exactMatch) {
        return normTitle === normQuery;
      }

      return normTitle.indexOf(normQuery) >= 0;
    };

    const domainListRenderer: ItemRenderer<DomainList> = (dl, { handleClick, handleFocus, modifiers }) => {
      if (!modifiers.matchesPredicate) {
        return null;
      }
      return (
        <MenuItem
            active={modifiers.active}
            disabled={modifiers.disabled}
            key={dl.id}
            labelElement={<DomainsSummary domains={dl.domains} />}
            onClick={handleClick}
            onFocus={handleFocus}
            roleStructure="listoption"
            text={dl.name}
        />
    );

    }

    const usedDomainListIds = new Set(client.rules?.flatMap(r => r.domainlists));
    const unusedDomainLists = domains.filter(dl => dl.id != null && !usedDomainListIds.has(dl.id));


    return (
      <Section title="Blocked domains">
        <SectionCard>
          {(client.rules === undefined || client.rules.length === 0) &&
            <p>
              No domains are currently blocked for {client.name}. Choose domains
              to block below and click add.
            </p>
          }
          {
            (client.rules) &&
            <Table columnNames={["Domain List", "Domains", ""]}>
              {client.rules.filter(r => r.kind === "deny_http_access").flatMap(r => r.domainlists).map(dlid =>
              {
                let domainList = domains.filter(dl => dl.id === dlid)[0];
                return (
                  <tr key={domainList.id}>
                    <td>{domainList.name}</td>
                    <td><DomainsSummary domains={domainList.domains} /></td>
                    <td><Button onClick={() => remove(domainList.id!)}><Remove /></Button></td>
                  </tr>
                );
              })
              }
            </Table>
          }

        </SectionCard>
        <SectionCard>
          <div css={css`display: grid; grid-template-columns: 1fr auto; grid-gap: 10px;`}>
          <MultiSelect<DomainList>
              popoverProps={{ minimal: true }}
              selectedItems={Array.from(selectedDomains)}
              items={unusedDomainLists}
              itemPredicate={filterDomainLists}
              itemRenderer={domainListRenderer}
              onItemSelect={(dl) => {
                const copy = new Set<DomainList>(selectedDomains);
                copy.add(dl);
                setSelectedDomains(copy);
              }}
              onRemove={(dl) => {
                const copy = new Set<DomainList>(selectedDomains);
                copy.delete(dl);
                setSelectedDomains(copy);
              }}
              onClear={() => setSelectedDomains(new Set())}
              tagRenderer={(dl) => <span>{dl.name}</span>}
          ></MultiSelect>

          <Button
              disabled={selectedDomains.size === 0}
              onClick={add}
          >Add</Button>

          </div>
        </SectionCard>
      </Section>
    );
  }

  const { domains } = useRouteLoaderData("root") as AppGridLoaderData;

  function BlockedDomains() {
    const [selected, setSelected] = useState<Set<DomainList>>(new Set());

    const addDls = async () => {
      if (client.rules === undefined) {
        client.rules = [];
      }
      client.rules.push({
        kind: "deny_http_access",
        domainlists: Array.from(selected).map((dl) => dl.id!)
      });

      (await updateClient(client)).andThen(revalidate);
    }

    const removeDl = async (dlid: number) => {
      const ruleIndex = client.rules?.findIndex(r => r.kind === "deny_http_access" && r.domainlists.includes(dlid));
      if (ruleIndex !== undefined) {
        const rule = client.rules[ruleIndex];

        const pos = rule.domainlists.findIndex(id => id === dlid)!;
        rule.domainlists.splice(pos, 1);
        if (rule.domainlists.length === 0) {
          client.rules.splice(ruleIndex, 1);
        }
        (await updateClient(client)).andThen(revalidate);
      } else {
        console.error(`dlid ${dlid} not found in `, client.rules);
      }
    }


    return <DomainListChooser
        domains={domains.unwrap()}
        selectedDomains={selected}
        setSelectedDomains={setSelected}
        add={addDls}
        remove={removeDl} />
  }

  const gridStyle = css`
    display: grid;
    grid-gap: 18px;
    grid-template-columns: 1fr 1fr;
  `;

  return (
    <div css={gridStyle}>
      <ClientDetails />
      <BlockedDomains />
    </div>
  )
}
