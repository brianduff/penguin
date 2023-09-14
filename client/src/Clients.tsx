import { useQuery } from "react-query"
import { getClients } from "./api"


export function Clients() {
  const query = useQuery("clients", getClients)

  return (
    <p>
      Clients:
      <ul css={{
        listStyle: 'none',
        paddingInlineStart: '0'
      }}>
        {query.data && query.data.map(client => (
          <li>{client.name} - {client.ip}</li>
        ))}
      </ul>
      <button>Add Client</button>
    </p>
  )
}
