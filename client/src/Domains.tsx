import { Section, SectionCard } from "@blueprintjs/core";
import { GlobeNetwork } from "@blueprintjs/icons";
import { Table } from "./components/Table";
import { createDomainList } from "./api";
import { useQueryClient } from "react-query";
import { InputWithButton } from "./components/InputWithButton";
import { useState } from "react";
import { Result } from "./result";
import { DomainList } from "./bindings/DomainList";
import { useRouteLoaderData } from "react-router-dom";
import { AppGridLoaderData } from "./main";

function validateDomainName(domainName: string): Result<string> {
  if (domainName.length <= 1) {
    return Result.Err("Domain name is too short");
  }

  let segments = domainName.split(".");
  if (segments.length < 2) {
    return Result.Err("Domain must have at least two components");
  }

  for (var i = 0; i < segments.length; i++) {
    const segment = segments[i];
    if (segment.length > 63) {
      return Result.Err(`Domain segment is too long: '${segment.slice(0, 10)}...'`)
    }
    if (segment.length === 0 && i > 0) {
      return Result.Err(`Empty domain segment not allowed`);
    }
    for (const c of segment) {
      if (isSymbol(c) && !"_-".includes(c)) {
        return Result.Err(`Domain includes invalid symbol: '${c}'`);
      }
    }
  }
  return Result.Ok(domainName);
}

function isSymbol(c: string) {
  return c.toLocaleUpperCase() === c.toLocaleLowerCase();
}

function generateNewDomainListName(lists: DomainList[]) {
  const prefix = "Domain List";
  let max = 0;
  for (const list of lists) {
    if (list.name === prefix) {
      max = 1;
    }
    if (list.name.startsWith(prefix)) {
      const suffix = list.name.substring(prefix.length);
      let suffixNumber = parseInt(suffix);
      if (!isNaN(suffixNumber)) {
        max = suffixNumber;
      }
    }
  }
  if (max > 0) {
    return prefix + " " + (max + 1);
  }
  return prefix;
}

export function Domains() {
  const { domains } = useRouteLoaderData("root") as AppGridLoaderData;
  const [newDomain, setNewDomain] = useState("");
  const [errorMessage, setErrorMessage] = useState("");
  const queryClient = useQueryClient();

  const addDomain = async () => {
    let result = await validateDomainName(newDomain)
        .andThen(submitDomainList);
    if (result.error) {
      setErrorMessage(result.error);
    }
  };
  const updateNewDomain = (value: string) => {
    for (const c of value) {
      if (isSymbol(c) && !"_-.".includes(c)) {
        return;
      }
    }

    if (value.length > 0 && value.charAt(0) !== '.') {
        value = '.' + value;
    }

    if (value.length > 254) {
      value = value.slice(0, 254);
    }

    value = value.toLocaleLowerCase();

    setNewDomain(value);
  };

  const submitDomainList = async (dl: string) => {
    let newName = "";
    if (domains.unwrap()) {
      newName = generateNewDomainListName(domains.unwrap());
    }

    return (await createDomainList({
      id: null,
      name: newName,
      domains: [ dl ]
    })).andThen(dl => {
      queryClient.invalidateQueries({ queryKey: [ "domainlists" ]})
      return Promise.resolve(Result.Ok(dl))
    })
  }

  return (
    <Section title="Sites" icon={<GlobeNetwork />}>
      <SectionCard>
        <Table columnNames={["Name", "Domains"]}>
          {domains.unwrap().map(list => (
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