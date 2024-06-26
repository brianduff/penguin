import { useLoaderData, useNavigate, useRevalidator, useRouteLoaderData } from "react-router-dom";
import { ErrorMessage } from "./components/ErrorMessage";
import { Result } from "./result";
import { Client } from "./bindings/Client";
import { Button, ButtonGroup, Dialog, DialogBody, DialogFooter, EditableText, MenuItem, Section, SectionCard, Switch } from "@blueprintjs/core";
import { css } from "@emotion/react";
import { Delete, Edit, Pause, Play, Remove } from "@blueprintjs/icons";
import { MultiSelect, ItemPredicate, ItemRenderer } from "@blueprintjs/select";
import { useRef, useState } from "react";
import { createNetAccess, deleteClient, getProxyLogs, updateClient, updateNetAccess } from "./api";
import { FieldEditor } from "./components/FieldEditor";
import { DomainList } from "./bindings/DomainList";
import { AppGridLoaderData, ViewClientLoaderData } from "./main";
import { DomainsSummary } from "./Domains";
import { Table } from "./components/Table";
import { SimpleSelect } from "./components/SimpleSelect";
import { Lease } from "./bindings/Lease";
import { NetAccess } from "./bindings/NetAccess";
import { gridStyle } from "./commonstyles";
import { useQuery } from "react-query";
import { LogEntry } from "./bindings/LogEntry";
import { RuleKind } from "./bindings/Rule";

export function ViewClient() {
  const data = useLoaderData() as Result<ViewClientLoaderData>;
  return data.match({
    ok: data => <div><Grid client={data.client.unwrap()} netaccess={data.netaccess?.unwrap()} /></div>,
    err: msg => <ErrorMessage message={msg} />
  })
}

interface Props {
  client: Client,
  netaccess?: NetAccess
}

function Grid({ client, netaccess }: Props) {

  const revalidator = useRevalidator();
  const revalidate = async (value: any) => {
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

    const navigateToClients = async (value: Client) => {
      navigate("/");
      return Result.Ok(value);
    }

    const onClickDelete = async () => {
      return (await (await deleteClient(client)).andThen(revalidate)).andThen(navigateToClients);
    }

    let proxyRequired = false;
    if (client.mac_address) {
      if (netaccess) {
        proxyRequired = !netaccess.enabled;
      }
    }

    const toggleProxyRequired = async () => {
      // We're toggling, so our new state is !proxyRequired.
      let newProxyRequired = !proxyRequired;

      let internetEnabled = !newProxyRequired;
      if (netaccess) {
        netaccess.enabled = internetEnabled;
        (await updateNetAccess(netaccess)).andThen(revalidate);
      } else {
        netaccess = {
          mac_address: client.mac_address!,
          enabled: internetEnabled
        };
        (await createNetAccess(netaccess)).andThen(revalidate);
      }
    };

    return (
      <Section title="Details" rightElement={
        <>
          <Button onClick={onClickDelete} icon={<Delete />} />
          <Button onClick={onClickEdit} icon={<Edit />} />
        </>
        }>
        <SectionCard>
          <div css={css`
                display: grid;
                grid-template-columns: auto 1fr;
                grid-gap: 10px;`
            }>
            <span>Name:</span>
            <FieldEditor onSubmit={commitClient} field="name" original={client} />
            <span>IP&nbsp;Address:</span>
            <FieldEditor onSubmit={commitClient} field="ip" original={client} />
            <span>Mac&nbsp;Address:</span>
            <FieldEditor onSubmit={commitClient} field="mac_address" original={client} />
            <span css={css`grid-column: 1 / 3;`}>
              <Switch
                  checked={proxyRequired}
                  disabled={client.mac_address == undefined}
                  onChange={toggleProxyRequired}
              >Proxy required to access internet</Switch>
            </span>
          </div>
        </SectionCard>
      </Section>
    );
  }


  const { domains } = useRouteLoaderData("root") as AppGridLoaderData;

  function BlockedDomains() {
    const [selected, setSelected] = useState<Set<DomainList>>(new Set());
    const [pauseDialogOpen, setPauseDialogOpen] = useState(false);
    const [pausingDl, setPausingDl] = useState<number | null>(null);

    const add = async () => {
      if (client.rules === undefined) {
        client.rules = [];
      }
      client.rules.push({
        kind: RuleKind.DENY_HTTP_ACCESS,
        domainlists: Array.from(selected).map((dl) => dl.id!)
      });

      (await updateClient(client)).andThen(revalidate);
    }

    const remove = async (dlid: number) => {
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

    const pause = async (dlid: number) => {
      setPausingDl(dlid);
      setPauseDialogOpen(true);
    }

    const closeDialog = (value: Client): Promise<Result<Client>> => {
      setPauseDialogOpen(false);
      return Promise.resolve(Result.Ok(value));
    }


    const savePause = async (time: PauseTime) => {
      const now = new Date().getTime();
      const end = now + (time.delta_mins! * 60 * 1000);  // TODO handle custom times

      if (client.leases === undefined) {
        client.leases = [];
      }
      client.leases.push({
        end_date: null,
        end_date_utc: new Date(end).getTime(),
        rule: {
          kind: RuleKind.ALLOW_HTTP_ACCESS,
          domainlists: [ pausingDl! ]
        }
      });

      (await (await updateClient(client)).andThen(revalidate)).andThen(closeDialog);
    }

    const resume = async (dl: DomainList) => {
      // Remove all leases for the given dl.
      let newLeases = client.leases?.filter(l => !l.rule.domainlists.includes(dl.id!));
      client.leases = newLeases;
      (await updateClient(client)).andThen(revalidate);
    }

    function DomainListChooser() {

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
      const unusedDomainLists = domains.unwrap().filter(dl => dl.id != null && !usedDomainListIds.has(dl.id));

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
              <Table columnNames={["Domain List", "Domains", "", ""]}>
                {client.rules.filter(r => r.kind === "deny_http_access").flatMap(r => r.domainlists).map(dlid =>
                {
                  let domainList = domains.unwrap().filter(dl => dl.id === dlid)[0];
                  return (
                    <tr key={domainList.id}>
                      <td>{domainList.name}</td>
                      <td><DomainsSummary domains={domainList.domains} /></td>
                      <td><UnblockStatus dl={domainList} /></td>
                      <td>
                        <ButtonGroup minimal={true}>
                          {(getActiveLeases(domainList).length === 0) &&
                          <Button onClick={() => pause(domainList.id!)}><Pause /></Button>}
                          {(getActiveLeases(domainList).length !== 0) &&
                          <Button onClick={() => resume(domainList)}><Play /></Button>}
                          <Button onClick={() => remove(domainList.id!)}><Remove /></Button>
                        </ButtonGroup>
                      </td>
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
                selectedItems={Array.from(selected)}
                items={unusedDomainLists}
                itemPredicate={filterDomainLists}
                itemRenderer={domainListRenderer}
                onItemSelect={(dl) => {
                  const copy = new Set<DomainList>(selected);
                  copy.add(dl);
                  setSelected(copy);
                }}
                onRemove={(dl) => {
                  const copy = new Set<DomainList>(selected);
                  copy.delete(dl);
                  setSelected(copy);
                }}
                onClear={() => setSelected(new Set())}
                tagRenderer={(dl) => <span>{dl.name}</span>}
            ></MultiSelect>

            <Button
                disabled={selected.size === 0}
                onClick={add}
            >Add</Button>

            </div>
          </SectionCard>
        </Section>
      );
    }

    interface UnblockStatusProps {
      dl: DomainList
    }

    function UnblockStatus({ dl }: UnblockStatusProps) {
      const leaseDates = getActiveLeases(dl)
          .flatMap(l => l.end_date_utc!)
          .map(d => new Date(d))
          .sort();
      if (leaseDates && leaseDates.length > 0) {
        const lastDate = leaseDates[leaseDates.length - 1];
        return <div css={css`color: #66ff66`}>Temporarily unblocked until {lastDate.toLocaleString()}</div>
      }

      return <span />
    }



    return (<>
      <DomainListChooser />
      <PauseDialog
          isOpen={pauseDialogOpen}
          setSelectedDate={savePause}
          close={() => setPauseDialogOpen(false)} />
    </>)
  }

  function getActiveLeases(dl: DomainList) {
    let leases = client.leases
      ?.filter(l => l.rule.kind === "allow_http_access")
      .filter(l => l.rule.domainlists.includes(dl.id!));

    if (leases === undefined) {
      return new Array<Lease>();
    }
    return leases;
  }

  function get_domain(logFqdn: string) {
    let parts = logFqdn.split("/");
    let fqdn = parts[parts.length - 1];
    let fqdn_parts = fqdn.split(".");

    let result = undefined;

    let tld = fqdn_parts[fqdn_parts.length - 1];
    if (tld === "com" || tld === "org" || tld === "net") {
      result = [fqdn_parts[fqdn_parts.length - 2], fqdn_parts[fqdn_parts.length - 1]].join(".");
    }

    fqdn_parts.splice(0, 1);
    result = fqdn_parts;

    if (result === undefined || result.length === 0) {
      console.log("Failed to parse fqdn", logFqdn);
    }
    return result.join(".");
  }

  function Logs() {
    const { isLoading, error, data: logs } = useQuery(["proxylogs", client.ip], () => getProxyLogs(client));

    console.log(error);

    // Group logs by day, then by domain.
    let groupedLogs = new Map<number, Map<string, Array<LogEntry>>>();

    if (logs && logs.isOk()) {
      let now = new Date();
      for (const entry of logs.unwrap()) {
        let days = Math.floor((now.getTime() - entry.date) / (1000 * 3600 * 24));
        let dayMap = groupedLogs.get(days);
        if (dayMap === undefined) {
          dayMap = new Map<string, Array<LogEntry>>();
          groupedLogs.set(days, dayMap);
        }

        let domain = get_domain(entry.peer_fqdn);
        let arr = dayMap.get(domain);
        if (arr === undefined) {
          arr = new Array<LogEntry>();
          dayMap.set(domain, arr);
        }
        arr.push(entry);
      }
    }


    return (
      <Section title="Logs">
        <SectionCard>
          {isLoading && <p>Loading...</p>}
          {logs && <p>I got {logs?.unwrap().length} log entries for this client!</p>}
          {new Array(...groupedLogs.entries()).map(([day, entries]) => (
            <li>{day} days ago<ul>
              {new Array(...entries).map(([domain, logEntries]) =>
                {
                  return (<li>{domain} - {logEntries.length} times</li>);
                }
              )}
              </ul>
            </li>
          ))}
        </SectionCard>
      </Section>
    )
  }

  return (
    <div css={gridStyle}>
      <BlockedDomains />
      <ClientDetails />
      <Logs />
    </div>
  )
}

interface PauseDialogProps {
  isOpen: boolean,
  close: () => void,
  setSelectedDate: (date: PauseTime) => void,
}

interface PauseTime {
  name: string,
  delta_mins?: number,
}

const PAUSE_TIMES : PauseTime[] = [
  {
    name: "30 minutes",
    delta_mins: 30
  },
  {
    name: "One hour",
    delta_mins: 60
  },
  {
    name: "Two hours",
    delta_mins: 120
  },
  {
    name: "Six hours",
    delta_mins: 6 * 60
  },
  {
    name: "One day",
    delta_mins: 24 * 60
  },
  // {
  //   name: "Custom"
  // }
]

function PauseDialog({ isOpen, setSelectedDate, close }: PauseDialogProps) {
  const [currentDate, setCurrentDate] = useState<null | PauseTime>(null);

  return (
    <Dialog title="Pause block" isOpen={isOpen} onClose={close}>
      <DialogBody>
        <p>
          The domains in this list will be unblocked for the specified time.
        </p>
        <p>
          <SimpleSelect<PauseTime>
              items={PAUSE_TIMES}
              render={t => t.name}
              setSelected={t => { setCurrentDate(t) }} />
        </p>
      </DialogBody>
      <DialogFooter minimal={true} actions={
        <Button disabled={currentDate === null} intent="primary" text="Unblock" onClick={() => setSelectedDate(currentDate!)} />
      }/>
    </Dialog>
  )
}