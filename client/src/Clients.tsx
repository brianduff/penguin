import { useQueryClient } from "react-query"
import { createClient } from "./api"
import { Icon, Section, SectionCard, Tooltip } from "@blueprintjs/core"
import { css } from '@emotion/react';
import { useState } from "react";
import { Desktop } from "@blueprintjs/icons";
import { Client } from "./bindings/Client";
import { InputWithButton } from "./components/InputWithButton";
import { Table } from "./components/Table";
import { Link, useRevalidator, useRouteLoaderData } from "react-router-dom";
import { AppGridLoaderData } from "./main";
import { DomainList } from "./bindings/DomainList";
import { Result } from "./result";

function validateIpAddress(text: string) {
  if (text.length == 0) {
    // It's always ok to clear the field
    return true;
  }
  // Every character in the text must be a numeric or a period
  let periodCount = 0;
  for (const c of text) {
    if (!"01234567890.".includes(c)) {
      return false;
    }
    if (c == '.') {
      periodCount++;
    }
  }
  // Each component of the ip address may be blank or must be
  // a value from 0-255.
  let parts = text.split('.');
  for (const part of parts) {
    if (part.length > 0) {
      let numeric = parseInt(part, 10);
      if (numeric > 255) {
        return false;
      }
      // Don't allow unnecessary leading zeros.
      if (numeric.toString() !== part) {
        return false;
      }
    }
  }

  // Don't allow periods with nothing in between
  if (text.includes('..') || text.includes('...')) {
    return false;
  }

  return periodCount <= 3;
}

export function Clients() {
  const { clients, domains } = useRouteLoaderData("root") as AppGridLoaderData;
  const [newIp, setNewIp] = useState("");
  const [errorMessage, setErrorMessage] = useState("");
  const queryClient = useQueryClient();
  const revalidator = useRevalidator();

  const leftAlign = css`
    text-align: left
  `;

  const updateNewIp = (newValue: string) => {
    if (validateIpAddress(newValue)) {
      setNewIp(newValue);
    }
  };

  async function addClient() {
    if (newIp.trim().length === 0) return;
    setErrorMessage("");
    let result = await createClient({
      id: null,
      ip: newIp,
      name: `Client with IP ${newIp}`,
      rules: [],
      leases: []
    })
    result.match({
      ok: (_) => {
        queryClient.invalidateQueries({ queryKey: ["clients"]})
        revalidator.revalidate();
        setNewIp("");
      },
      err: (err) => {
        setErrorMessage(err);
      }
    })

  }

  return (
    <>
      <Section title="Computers" icon={<Desktop />}>
        <SectionCard>
          <Table columnNames={["Address", "Name", "Blocked domains"]}>
            {clients.unwrap().map(client => (
              <tr key={client.ip}>
                <td css={leftAlign}><Link to={`/client/${client.id}`}>{client.ip}</Link></td>
                <td>{client.name}</td>
                <td><BlockedDomainCount client={client} domains={domains} /></td>
              </tr>
            ))}
          </Table>
        </SectionCard>
        <SectionCard>
          <InputWithButton
              prompt="IP address of computer"
              value={newIp}
              submit={addClient}
              onValueUpdated={updateNewIp}
              errorMessage={errorMessage}
              setErrorMessage={setErrorMessage} />
        </SectionCard>
      </Section>
    </>
  )
}

interface BlockedDomainCountProps {
  client: Client,
  domains: Result<DomainList[]>,
}

function BlockedDomainCount({ client, domains }: BlockedDomainCountProps) {

  // Get all the blocked domain lists
  let domainlists = new Set(client.rules
      ?.filter(r => r.kind === "deny_http_access")
      .flatMap(rule => rule.domainlists));

  // Remove any that are currently temporarily leased. We trust
  // that the server is pruning expired leases.
  client.leases
      ?.filter(l => l.rule.kind === "allow_http_access")
      .flatMap(l => l.rule.domainlists)
      ?.forEach(id => domainlists.delete(id));

  return domains.match({
    ok: lists => {
      let domains = new Set<string>();
      for (const dl of lists) {
        if (dl.id !== null && domainlists.has(dl.id)) {
          dl.domains?.forEach(d => domains.add(d));
        }
      }
      return (<span>{domains.size}</span>);
    },
    err: error => {
      return (<Tooltip content={`Can't retrieve domains: ${error}`}>
        <span><Icon icon="warning-sign" /></span>
      </Tooltip>);
    }
  })
}