import { useQuery, useQueryClient } from "react-query"
import { createClient, getClients } from "./api"
import { Button, HTMLTable, InputGroup, Section, SectionCard } from "@blueprintjs/core"
import { css } from '@emotion/react';
import { useState } from "react";
import { Desktop } from "@blueprintjs/icons";

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
  const query = useQuery("clients", getClients)
  const queryClient = useQueryClient();

  const leftAlign = css`
    text-align: left
  `;

  const updateNewIp = (e: React.ChangeEvent<HTMLInputElement>) => {
    if (validateIpAddress(e.target.value)) {
      setNewIp(e.target.value);
    }
  };

  async function addClient(newIp: string) {
    await createClient({
      id: null,
      ip: newIp,
      name: `Client with IP ${newIp}`,
      rules: [],
      leases: []
    });

    queryClient.invalidateQueries({ queryKey: ["clients"]})
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
              </tr>
            </thead>
            <tbody>
              {query.data && query.data.map(client => (
                <tr key={client.ip}>
                  <td css={leftAlign}>{client.ip}</td>
                  <td>{client.name}</td>
                </tr>
              ))}
            </tbody>
          </HTMLTable>
        </SectionCard>
        <SectionCard>
          <InputGroup
              value={newIp}
              className="pt-input"
              placeholder="IP address of new client"
              rightElement={
                <Button disabled={newIp.trim().length == 0}
                    onClick={() => addClient(newIp)}
                    minimal={true}
                    intent="primary">Add
                </Button>}
              onChange={updateNewIp} />
        </SectionCard>
      </Section>
    </>
  )
}
