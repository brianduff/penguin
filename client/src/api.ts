import { Client } from "./bindings/Client";

async function get<T>(path: string) {
  return await req<T>(path, "GET", undefined);
}

async function req<T>(path: string, method: string, body: any) {
  let options: RequestInit = {
    method,
  };
  if (body) {
    options.headers = {
      "Content-Type": "application/json"
    }
    options.body = JSON.stringify(body);
  }

  let result = await fetch(`http://localhost:8080/api/v1/${path}`, options);
  let json = await result.json();
  return json as T;
}

export async function getClients() {
  return get<Array<Client>>("client")
}

export async function createClient(client: Client) {
  return req<Client>("client", "POST", client)
}