import React from 'react'
import ReactDOM from 'react-dom/client'
import App, { AppGrid } from './App.tsx'
import './index.css'
import { ViewClient } from './ViewClient.tsx'
import { RouterProvider, createBrowserRouter } from 'react-router-dom'
import { QueryClient, QueryClientProvider } from 'react-query'
import { getClient, getClients, getDomainList, getDomainLists } from './api.ts'
import { Result } from './result.ts'
import { DomainList } from './bindings/DomainList.ts'
import { Client } from './bindings/Client.ts'
import { Desktop, Globe, GlobeNetwork, Home } from '@blueprintjs/icons'
import { ViewDomains } from './ViewDomains.tsx'

const queryClient = new QueryClient();

export interface AppGridLoaderData {
  clients: Result<Client[]>,
  domains: Result<DomainList[]>
}

const router = createBrowserRouter([
  {
    id: "root",
    path: "/",
    loader: async () => {
      const [clients, domains] = await Promise.all([
        queryClient.fetchQuery("clients", getClients),
        queryClient.fetchQuery("domainlists", getDomainLists)
      ]);

      return { clients, domains }
    },
    handle: {
      crumb: (_: any) => ({ href: "/", text: "Home", icon: <Home />})
    },
    element: <App />,
    children: [
      {
        path: "/",
        element: <AppGrid />,
        id: "appgrid",
      },
      {
        id: "client",
        path: "client/:id",
        element: <ViewClient />,
        loader: async ({ params }) => {
          return queryClient.fetchQuery("client", () => getClient(params.id!))
        },
        handle: {
          crumb: (data: any) => {
            let client = (data as Result<Client>).unwrap();
            return ({ href: `/client/${client.id}`, text: client.name, icon: <Desktop />})
          }
        }
      },
      {
        id: "domains",
        path: "domains/:id",
        element: <ViewDomains />,
        loader: async ({ params }) => {
          return queryClient.fetchQuery("domain", () => getDomainList(params.id!))
        },
        handle: {
          crumb: (data: any) => {
            let domainList = (data as Result<DomainList>).unwrap();
            return ({ href: `/client/${domainList.id}`, text: domainList.name, icon: <GlobeNetwork />})
          }
        }
      }
    ]
  }
])


ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <QueryClientProvider client={queryClient}>
      <RouterProvider router={router} />
    </QueryClientProvider>
  </React.StrictMode>,
)
