import { Client } from "./bindings/Client";
import { DomainList } from "./bindings/DomainList";
import { Result } from "./result";

async function get<T>(path: string) {
  return await req<T>(path, "GET", undefined);
}

async function req<T>(path: string, method: string, body: any) : Promise<Result<T>> {
  let options: RequestInit = {
    method,
  };
  if (body) {
    options.headers = {
      "Content-Type": "application/json"
    }
    options.body = JSON.stringify(body);
  }

  console.log(`${method} ${path}`)
  let result = await fetch(`http://localhost:8080/api/v1/${path}`, options);

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

export async function getDomainLists() {
  return get<Array<DomainList>>("domainlist")
}

export async function createDomainList(domainList: DomainList) {
  return req<DomainList>("domainlist", "POST", domainList)
}