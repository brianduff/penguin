import { useQuery, useQueryClient } from "react-query"
import { createClient, getClients, getDomainLists } from "./api"
import { Button, Callout, HTMLTable, Icon, InputGroup, Popover, Section, SectionCard, Tooltip } from "@blueprintjs/core"
import { css } from '@emotion/react';
import { useState } from "react";
import { Desktop } from "@blueprintjs/icons";
import { chain, onKey } from "./events";
import { Client } from "./bindings/Client";
import { InputWithButton } from "./components/InputWithButton";

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
  const [newIp, setNewIp] = useState("");
  const [errorMessage, setErrorMessage] = useState("");
  const query = useQuery("clients", getClients)
  const queryClient = useQueryClient();

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
        setNewIp("");
      },
      err: (err) => {
        setErrorMessage(err);
      }
    })

  }

  return (
    <>
      <Section title="Clients" icon={<Desktop />}>
        <SectionCard>
          <HTMLTable compact={true} striped={true}>
            <thead>
              <tr>
                <th>Address</th>
                <th>Name</th>
                <th>Blocked domains</th>
              </tr>
            </thead>
            <tbody>
              {query.data && query.data.isOk() && query.data.unwrap().map(client => (
                <tr key={client.ip}>
                  <td css={leftAlign}>{client.ip}</td>
                  <td>{client.name}</td>
                  <td><BlockedDomainCount client={client} /></td>
                </tr>
              ))}
            </tbody>
          </HTMLTable>
        </SectionCard>
        <SectionCard>
          <InputWithButton
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
  client: Client
}

function BlockedDomainCount({ client }: BlockedDomainCountProps) {
  const query = useQuery("domainlists", getDomainLists);

  // Get all the blocked domain lists
  let domainlists = new Set(client.rules
      ?.filter(r => r.kind === "deny_http_access")
      .flatMap(rule => rule.domainlists));

  // Remove any that are currently temporarily leased. We trust
  // that the server is pruning expired leases.
  client.leases
      ?.filter(l => l.rule.kind === "allow_http_access")
      .flatMap(l => l.rule.domainlists)
      .forEach(id => domainlists.delete(id));

  if (query.data) {
    return query.data.match({
      ok: lists => {
        let domains = new Set<string>();
        for (const dl of lists) {
          if (dl.id !== null && domainlists.has(dl.id)) {
            dl.domains.forEach(d => domains.add(d));
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
  return <></>
}