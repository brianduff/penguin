import React from 'react'
import ReactDOM from 'react-dom/client'
import App, { AppGrid } from './App.tsx'
import './index.css'
import { ViewClient } from './Client.tsx'
import { RouterProvider, createBrowserRouter } from 'react-router-dom'
import { QueryClient, QueryClientProvider } from 'react-query'
import { getClients, getDomainLists } from './api.ts'
import { Result } from './result.ts'
import { DomainList } from './bindings/DomainList.ts'
import { Client } from './bindings/Client.ts'

const queryClient = new QueryClient();

export interface AppGridLoaderData {
  clients: Result<Client[]>,
  domains: Result<DomainList[]>
}

const router = createBrowserRouter([
  {
    path: "/",
    element: <App />,
    children: [
      {
        path: "/",
        element: <AppGrid />,
        loader: async () => {
          const [clients, domains] = await Promise.all([
            queryClient.fetchQuery("clients", getClients),
            queryClient.fetchQuery("domainlists", getDomainLists)
          ]);

          return { clients, domains }
        }
      },
      {
        path: "client/:id",
        element: <ViewClient />
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
