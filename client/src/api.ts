import { Client } from "./bindings/Client";
import { DomainList } from "./bindings/DomainList";
import { NetAccess } from "./bindings/NetAccess";
import { LogEntry } from "./bindings/LogEntry";
import { Result } from "./result";
import { getGoogleCredential } from "./components/GoogleAuth";
import { ServiceStatus } from "./bindings/ServiceStatus";

var BASE_URL = "http://localhost:8080/";
if (import.meta.env.PROD) {
  BASE_URL = "/"
}

async function get<T>(path: string) {
  return await req<T>(path, "GET", undefined);
}

async function req<T>(path: string, method: string, body: any) : Promise<Result<T>> {
  let options: RequestInit = {
    method,
  };
  let headers : { [key: string]: string } = {};

  let token = getGoogleCredential();
  if (token) {
    headers["Authorization"] = `Bearer ${token}`;
  }

  if (body) {
    headers["Content-Type"] = "application/json";
    options.body = JSON.stringify(body);
  }

  options.headers = headers;

  console.log(`${method} ${path}`, body)
  let result = await fetch(`${BASE_URL}api/v1/${path}`, options);

  console.log("Result: ", result);
  if (result.status != 200) {
    console.log(result);
    return Result.Err(await result.text());
  }

  let json = await result.json();
  return Result.Ok(json as T);
}

export async function getClients() {
  return get<Array<Client>>("client")
}

export async function getClient(id: string) {
  return get<Client>(`client/${id}`)
}

export async function createClient(client: Client) {
  return req<Client>("client", "POST", client)
}

export async function updateClient(client: Client) {
  return req<Client>(`client/${client.id}`, "PUT", client)
}

export async function deleteClient(client: Client) {
  return req<Client>(`client/${client.id}`, "DELETE", client)
}

export async function getDomainLists() {
  return get<Array<DomainList>>("domainlist")
}

export async function getDomainList(id: string) {
  return get<Client>(`domainlist/${id}`)
}

export async function createDomainList(domainList: DomainList) {
  return req<DomainList>("domainlist", "POST", domainList)
}

export async function updateDomainList(domainList: DomainList) {
  return req<DomainList>(`domainlist/${domainList.id}`, "PUT", domainList)
}

export async function getNetAccess(mac: string) {
  return get<NetAccess>(`netaccess/${mac}`)
}

export async function updateNetAccess(netAccess: NetAccess) {
  return req<NetAccess>(`netaccess/${netAccess.mac_address}`, "PUT", netAccess);
}

export async function createNetAccess(netAccess: NetAccess) {
  return req<NetAccess>("netaccess", "POST", netAccess);
}

export async function getProxyLogs(client: Client) {
  return get<Array<LogEntry>>(`logs/proxy?client_id=${client.id}`)
}

export async function getProxyStatus() {
  return get<ServiceStatus>("proxy")
}
